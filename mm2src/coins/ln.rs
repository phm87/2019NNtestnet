use super::{HistorySyncState, MarketCoinOps, MmCoin, SwapOps, TradeFee, TransactionDetails, TransactionEnum,
            TransactionFut};
use crate::{FoundSwapTxSpend, WithdrawRequest};
use bigdecimal::BigDecimal;
use common::mm_ctx::MmArc;
use futures01::Future;
use mocktopus::macros::*;

use crate::utxo::TxFee;
use crate::utxo::ActualTxFee;
use keys::{Address, KeyPair, Private, Public, Secret, Type};
use script::{Builder, Opcode, Script, ScriptAddress, SignatureVersion, TransactionInputSigner,
             UnsignedTransactionInput};

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering as AtomicOrderding};
use std::sync::{Arc, Mutex, Weak};
pub use bitcrypto::{dhash160, sha256, ChecksumType};
use futures::core_reexport::num::NonZeroU64;
use crate::utxo::rpc_clients::EstimateFeeMode;
use common::jsonrpc_client::JsonRpcError;
use crate::utxo::SWAP_TX_SPEND_SIZE;
use crate::utxo::KILO_BYTE;
use crate::utxo::sat_from_big_decimal;
use serialization::{deserialize, serialize};
use crate::utxo::payment_script;
pub use chain::Transaction as UtxoTx;
use std::ops::Deref;

/// Dummy coin struct used in tests which functions are unimplemented but then mocked
/// in specific test to emulate the required behavior
//#[derive(Clone, Debug)]
//pub struct LnCoin {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "format")]
enum LnAddressFormat {
    /// Standard UTXO address format.
    /// In Bitcoin Cash context the standard format also known as 'legacy'.
    #[serde(rename = "standard")]
    Standard,
    /// Bitcoin Cash specific address format.
    /// https://github.com/bitcoincashorg/bitcoincash.org/blob/master/spec/cashaddr.md
    #[serde(rename = "cashaddress")]
    CashAddress { network: String },
}

impl Default for LnAddressFormat {
    fn default() -> Self { LnAddressFormat::Standard }
}

/// pImpl idiom.
#[derive(Debug)]
pub struct LnCoinImpl {
    ticker: String,
    /// https://en.bitcoin.it/wiki/List_of_address_prefixes
    /// https://github.com/jl777/coins/blob/master/coins
    pub_addr_prefix: u8,
    p2sh_addr_prefix: u8,
    wif_prefix: u8,
    pub_t_addr_prefix: u8,
    p2sh_t_addr_prefix: u8,
    /// True if coins uses Proof of Stake consensus algo
    /// Proof of Work is expected by default
    /// https://en.bitcoin.it/wiki/Proof_of_Stake
    /// https://en.bitcoin.it/wiki/Proof_of_work
    /// The actual meaning of this is nTime field is used in transaction
    is_pos: bool,
    /// Special field for Zcash and it's forks
    /// Defines if Overwinter network upgrade was activated
    /// https://z.cash/upgrade/overwinter/
    overwintered: bool,
    /// The tx version used to detect the transaction ser/de/signing algo
    /// For now it's mostly used for Zcash and forks because they changed the algo in
    /// Overwinter and then Sapling upgrades
    /// https://github.com/zcash/zips/blob/master/zip-0243.rst
    tx_version: i32,
    /// If true - allow coins withdraw to P2SH addresses (Segwit).
    /// the flag will also affect the address that MM2 generates by default in the future
    /// will be the Segwit (starting from 3 for BTC case) instead of legacy
    /// https://en.bitcoin.it/wiki/Segregated_Witness
    segwit: bool,
    /// Default decimals amount is 8 (BTC and almost all other UTXO coins)
    /// But there are forks which have different decimals:
    /// Peercoin has 6
    /// Emercoin has 6
    /// Bitcoin Diamond has 7
    decimals: u8,
    /// Does coin require transactions to be notarized to be considered as confirmed?
    /// https://komodoplatform.com/security-delayed-proof-of-work-dpow/
    requires_notarization: AtomicBool,
    /// RPC client
    rpc_client: UtxoRpcClientEnum,
    /// ECDSA key pair
    key_pair: KeyPair,
    /// Lock the mutex when we deal with address utxos
    my_address: Address,
    /// The address format indicates how to parse and display UTXO addresses over RPC calls
    address_format: LnAddressFormat,
    /// Is current coin KMD asset chain?
    /// https://komodoplatform.atlassian.net/wiki/spaces/KPSD/pages/71729160/What+is+a+Parallel+Chain+Asset+Chain
    asset_chain: bool,
    tx_fee: TxFee,
    /// Transaction version group id for Zcash transactions since Overwinter: https://github.com/zcash/zips/blob/master/zip-0202.rst
    version_group_id: u32,
    /// Consensus branch id for Zcash transactions since Overwinter: https://github.com/zcash/zcash/blob/master/src/consensus/upgrades.cpp#L11
    /// used in transaction sig hash calculation
    consensus_branch_id: u32,
    /// Defines if coin uses Zcash transaction format
    zcash: bool,
    /// Address and privkey checksum type
    checksum_type: ChecksumType,
    /// Fork id used in sighash
    fork_id: u32,
    /// Signature version
    signature_version: SignatureVersion,
    history_sync_state: Mutex<HistorySyncState>,
    required_confirmations: AtomicU64,
    /// if set to true MM2 will check whether calculated fee is lower than relay fee and use
    /// relay fee amount instead of calculated
    /// https://github.com/KomodoPlatform/atomicDEX-API/issues/617
    force_min_relay_fee: bool,
    /// Block count for median time past calculation
    mtp_block_count: NonZeroU64,
    estimate_fee_mode: Option<EstimateFeeMode>,
}

impl LnCoinImpl {
    async fn get_tx_fee(&self) -> Result<ActualTxFee, JsonRpcError> {
        match &self.tx_fee {
            TxFee::Fixed(fee) => Ok(ActualTxFee::Fixed(*fee)),
            TxFee::Dynamic(method) => {
                let fee = self
                    .rpc_client
                    .estimate_fee_sat(self.decimals, method, &self.estimate_fee_mode)
                    .compat()
                    .await?;
                Ok(ActualTxFee::Dynamic(fee))
            },
        }
    }

    /// returns the fee required to be paid for HTLC spend transaction
    async fn get_htlc_spend_fee(&self) -> Result<u64, String> {
        let coin_fee = try_s!(self.get_tx_fee().await);
        let mut fee = match coin_fee {
            ActualTxFee::Fixed(fee) => fee,
            // atomic swap payment spend transaction is slightly more than 300 bytes in average as of now
            ActualTxFee::Dynamic(fee_per_kb) => (fee_per_kb * SWAP_TX_SPEND_SIZE) / KILO_BYTE,
        };
        if self.force_min_relay_fee {
            let relay_fee = try_s!(self.rpc_client.get_relay_fee().compat().await);
            let relay_fee_sat = try_s!(sat_from_big_decimal(&relay_fee, self.decimals));
            if fee < relay_fee_sat {
                fee = relay_fee_sat;
            }
        }
        Ok(fee)
    }

    fn addresses_from_script(&self, script: &Script) -> Result<Vec<Address>, String> {
        let destinations: Vec<ScriptAddress> = try_s!(script.extract_destinations());

        let addresses = destinations
            .into_iter()
            .map(|dst| {
                let (prefix, t_addr_prefix) = match dst.kind {
                    Type::P2PKH => (self.pub_addr_prefix, self.pub_t_addr_prefix),
                    Type::P2SH => (self.p2sh_addr_prefix, self.p2sh_t_addr_prefix),
                };

                Address {
                    hash: dst.hash,
                    checksum_type: self.checksum_type,
                    prefix,
                    t_addr_prefix,
                }
            })
            .collect();

        Ok(addresses)
    }

    pub fn denominate_satoshis(&self, satoshi: i64) -> f64 { satoshi as f64 / 10f64.powf(self.decimals as f64) }

    fn search_for_swap_tx_spend(
        &self,
        time_lock: u32,
        first_pub: &Public,
        second_pub: &Public,
        secret_hash: &[u8],
        tx: &[u8],
        search_from_block: u64,
    ) -> Result<Option<FoundSwapTxSpend>, String> {
        let tx: UtxoTx = try_s!(deserialize(tx).map_err(|e| ERRL!("{:?}", e)));
        let script = payment_script(time_lock, secret_hash, first_pub, second_pub);
        let expected_script_pubkey = Builder::build_p2sh(&dhash160(&script)).to_bytes();
        if tx.outputs[0].script_pubkey != expected_script_pubkey {
            return ERR!(
                "Transaction {:?} output 0 script_pubkey doesn't match expected {:?}",
                tx,
                expected_script_pubkey
            );
        }

        let spend = try_s!(self.rpc_client.find_output_spend(&tx, 0, search_from_block).wait());
        match spend {
            Some(tx) => {
                let script: Script = tx.inputs[0].script_sig.clone().into();
                if let Some(Ok(ref i)) = script.iter().nth(2) {
                    if i.opcode == Opcode::OP_0 {
                        return Ok(Some(FoundSwapTxSpend::Spent(tx.into())));
                    }
                }

                if let Some(Ok(ref i)) = script.iter().nth(1) {
                    if i.opcode == Opcode::OP_1 {
                        return Ok(Some(FoundSwapTxSpend::Refunded(tx.into())));
                    }
                }

                ERR!(
                    "Couldn't find required instruction in script_sig of input 0 of tx {:?}",
                    tx
                )
            },
            None => Ok(None),
        }
    }

    pub fn my_public_key(&self) -> &Public { self.key_pair.public() }

    pub fn rpc_client(&self) -> &UtxoRpcClientEnum { &self.rpc_client }

    pub fn display_address(&self, address: &Address) -> Result<String, String> {
        match &self.address_format {
            LnAddressFormat::Standard => Ok(address.to_string()),
            LnAddressFormat::CashAddress { network } => address
                .to_cashaddress(&network, self.pub_addr_prefix, self.p2sh_addr_prefix)
                .and_then(|cashaddress| cashaddress.encode()),
        }
    }

    async fn get_current_mtp(&self) -> Result<u32, String> {
        let current_block = try_s!(self.rpc_client.get_block_count().compat().await);
        self.rpc_client
            .get_median_time_past(current_block, self.mtp_block_count)
            .compat()
            .await
    }
}

#[derive(Clone, Debug)]
pub struct LnCoin(Arc<LnCoinImpl>);
impl Deref for LnCoin {
    type Target = LnCoinImpl;
    fn deref(&self) -> &LnCoinImpl { &*self.0 }
}

impl From<LnCoinImpl> for LnCoin {
    fn from(coin: LnCoinImpl) -> LnCoin { LnCoin(Arc::new(coin)) }
}


//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////


#[mockable]
#[allow(clippy::forget_ref, clippy::forget_copy)]
impl MarketCoinOps for LnCoin {
    fn ticker(&self) -> &str { &self.ticker[..] }

    fn my_address(&self) -> Result<String, String> { unimplemented!() }

    fn my_balance(&self) -> Box<dyn Future<Item = BigDecimal, Error = String> + Send> { unimplemented!()
            // API REST call to LND for onchain balance: https://localhost:8080/v1/balance/blockchain
            // /v1/balance/channels
            // https://api.lightning.community/#v1-balance-channels
            }

    fn base_coin_balance(&self) -> Box<dyn Future<Item = BigDecimal, Error = String> + Send> { unimplemented!() }

    /// Receives raw transaction bytes in hexadecimal format as input and returns tx hash in hexadecimal format
    fn send_raw_tx(&self, tx: &str) -> Box<dyn Future<Item = String, Error = String> + Send> { unimplemented!() }

    fn wait_for_confirmations(
        &self,
        tx: &[u8],
        confirmations: u64,
        requires_nota: bool,
        wait_until: u64,
        check_every: u64,
    ) -> Box<dyn Future<Item = (), Error = String> + Send> {
        unimplemented!()
    }

    fn wait_for_tx_spend(&self, transaction: &[u8], wait_until: u64, from_block: u64) -> TransactionFut {
        unimplemented!()
    }

    fn tx_enum_from_bytes(&self, bytes: &[u8]) -> Result<TransactionEnum, String> { unimplemented!() }

    fn current_block(&self) -> Box<dyn Future<Item = u64, Error = String> + Send> { unimplemented!() }

    fn address_from_pubkey_str(&self, pubkey: &str) -> Result<String, String> { unimplemented!() }

    fn display_priv_key(&self) -> String { unimplemented!() }
}

#[mockable]
#[allow(clippy::forget_ref, clippy::forget_copy)]
impl SwapOps for LnCoin {
    fn send_taker_fee(&self, fee_addr: &[u8], amount: BigDecimal) -> TransactionFut { unimplemented!() }

    fn send_maker_payment(
        &self,
        time_lock: u32,
        taker_pub: &[u8],
        secret_hash: &[u8],
        amount: BigDecimal,
    ) -> TransactionFut {
        unimplemented!()
    }

    fn send_taker_payment(
        &self,
        time_lock: u32,
        maker_pub: &[u8],
        secret_hash: &[u8],
        amount: BigDecimal,
    ) -> TransactionFut {
        unimplemented!()
    }

    fn send_maker_spends_taker_payment(
        &self,
        taker_payment_tx: &[u8],
        time_lock: u32,
        taker_pub: &[u8],
        secret: &[u8],
    ) -> TransactionFut {
        unimplemented!()
    }

    fn send_taker_spends_maker_payment(
        &self,
        maker_payment_tx: &[u8],
        time_lock: u32,
        maker_pub: &[u8],
        secret: &[u8],
    ) -> TransactionFut {
        unimplemented!()
    }

    fn send_taker_refunds_payment(
        &self,
        taker_payment_tx: &[u8],
        time_lock: u32,
        maker_pub: &[u8],
        secret_hash: &[u8],
    ) -> TransactionFut {
        unimplemented!()
    }

    fn send_maker_refunds_payment(
        &self,
        maker_payment_tx: &[u8],
        time_lock: u32,
        taker_pub: &[u8],
        secret_hash: &[u8],
    ) -> TransactionFut {
        unimplemented!()
    }

    fn validate_fee(
        &self,
        fee_tx: &TransactionEnum,
        fee_addr: &[u8],
        amount: &BigDecimal,
    ) -> Box<dyn Future<Item = (), Error = String> + Send> {
        unimplemented!()
    }

    fn validate_maker_payment(
        &self,
        payment_tx: &[u8],
        time_lock: u32,
        maker_pub: &[u8],
        priv_bn_hash: &[u8],
        amount: BigDecimal,
    ) -> Box<dyn Future<Item = (), Error = String> + Send> {
        unimplemented!()
    }

    fn validate_taker_payment(
        &self,
        payment_tx: &[u8],
        time_lock: u32,
        taker_pub: &[u8],
        priv_bn_hash: &[u8],
        amount: BigDecimal,
    ) -> Box<dyn Future<Item = (), Error = String> + Send> {
        unimplemented!()
    }

    fn check_if_my_payment_sent(
        &self,
        time_lock: u32,
        other_pub: &[u8],
        secret_hash: &[u8],
        search_from_block: u64,
    ) -> Box<dyn Future<Item = Option<TransactionEnum>, Error = String> + Send> {
        unimplemented!()
    }

    fn search_for_swap_tx_spend_my(
        &self,
        time_lock: u32,
        other_pub: &[u8],
        secret_hash: &[u8],
        tx: &[u8],
        search_from_block: u64,
    ) -> Result<Option<FoundSwapTxSpend>, String> {
        unimplemented!()
    }

    fn search_for_swap_tx_spend_other(
        &self,
        time_lock: u32,
        other_pub: &[u8],
        secret_hash: &[u8],
        tx: &[u8],
        search_from_block: u64,
    ) -> Result<Option<FoundSwapTxSpend>, String> {
        unimplemented!()
    }
}

#[mockable]
#[allow(clippy::forget_ref, clippy::forget_copy)]
impl MmCoin for LnCoin {
    fn is_asset_chain(&self) -> bool { unimplemented!() }

    fn can_i_spend_other_payment(&self) -> Box<dyn Future<Item = (), Error = String> + Send> { unimplemented!() }

    fn withdraw(&self, req: WithdrawRequest) -> Box<dyn Future<Item = TransactionDetails, Error = String> + Send> {
        unimplemented!()
    }

    fn decimals(&self) -> u8 { unimplemented!() }

    fn process_history_loop(&self, ctx: MmArc) { unimplemented!() }

    fn tx_details_by_hash(&self, hash: &[u8]) -> Box<dyn Future<Item = TransactionDetails, Error = String> + Send> {
        unimplemented!()
    }

    fn history_sync_status(&self) -> HistorySyncState { unimplemented!() }

    /// Get fee to be paid per 1 swap transaction
    fn get_trade_fee(&self) -> Box<dyn Future<Item = TradeFee, Error = String> + Send> { unimplemented!() }

    fn required_confirmations(&self) -> u64 { 1 }

    fn requires_notarization(&self) -> bool { false }

    fn set_required_confirmations(&self, _confirmations: u64) { unimplemented!() }

    fn set_requires_notarization(&self, _requires_nota: bool) { unimplemented!() }
}

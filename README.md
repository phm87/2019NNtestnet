# How to partcipate
Fork this repo.

Edit the `iguana/m_notary_testnet` file. Add a line with your IP address. 

Edit the `iguana/testnet.json` file. Add your pubkey and name.

Make a PR with edits to this repo. 

# How to start the notary

Clone this repo.

Install the dependencies
```
sudo apt-get install build-essential pkg-config libc6-dev m4 \
        g++-multilib autoconf libtool ncurses-dev unzip git python \
        zlib1g-dev wget bsdmainutils automake libboost-all-dev \
        libssl-dev libprotobuf-dev protobuf-compiler \
        libqrencode-dev libdb++-dev ntp ntpdate vim software-properties-common \
        curl libevent-dev libcurl4-gnutls-dev libsodium-dev cmake clang
```

You must install nanomsg as well. See https://github.com/KomodoPlatform/komodo/wiki/Installing-Komodo-Manually#install-nanomsg

Start PIZZA, BEER and KMD daemons with `-pubkey=` 

Open p2p ports for each coin. Open port 17711 for iguana. 

Fund `-pubkey=` address on all 3 nodes. Import privkey to all 3 nodes. 

If you need PIZZA or BEER, use the faucets at https://www.atomicexplorer.com/#/faucet/beer and https://www.atomicexplorer.com/#/faucet/pizza or ask in #notarynode channel. 

Create a file named `pubkey.txt` at `~/2019NNtestnet/iguana/pubkey.txt`. Contents should be 
```
pubkey=<pubkey>
```

Create a file named `passphrase.txt` at `~/2019NNtestnet/iguana/passphrase.txt`. Contents should be
```
passphrase=<WIF>
```

Wait until the PR is merged. 

Then use the following to start notarization.
```
cd ~/2019NNtestnet/iguana
./m_notary_testnet
```

# How to restart when new participants are added 

```
pkill -15 iguana
./m_notary_testnet
```

## Details

While komodo is syncing, you can extract the privatekey (to keep secret) and the pubkey of the address of the account "":

```
./komodo-cli listaccounts
{
  "": 0.00000000
}

./komodo-cli getaccountaddress ""
RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx

./komodo-cli validateaddress RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx
{
  "isvalid": true,
  "address": "RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx",
  "scriptPubKey": "76a91454aa07657a7fb6ba864d550d78e158859dae451788ac",
  "segid": 63,
  "ismine": true,
  "iswatchonly": false,
  "isscript": false,
  "pubkey": "024529af3f14a307baff487a5ce9b4589debfb83d4d4b1dfb283c4967ea17f34ea",
  "iscompressed": true,
  "account": ""
}

./komodo-cli dumpprivkey RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx
xxxxx (to keep secret)
```

The commands to launch and sync BEER and PIZZA are available at:
https://github.com/jl777/komodo/blob/dev/src/assetchains.old#L26

```
./komodod -pubkey=024529af3f14a307baff487a5ce9b4589debfb83d4d4b1dfb283c4967ea17f34ea -ac_name=BEER -ac_supply=100000000 -addnode=78.47.196.146 -server -daemon
./komodod -pubkey=024529af3f14a307baff487a5ce9b4589debfb83d4d4b1dfb283c4967ea17f34ea -ac_name=PIZZA -ac_supply=100000000 -addnode=78.47.196.146 -server -daemon
```

Import the private key into BEER and PIZZA:

```
./komodo-cli -ac_name=BEER importprivkey xxxxx
RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx

./komodo-cli -ac_name=PIZZA importprivkey xxxxx
RGzrXSj52MuszgMRhkibXDvZ6xmUq5SiTx
```

Note your server public IP:

```
curl https://ipinfo.io/ip
```

Check that your three nodes are synced by comparing blockheight of your node with official block explorer:

https://kmdexplorer.io/
https://beer.explorer.dexstats.info/
https://pizza.explorer.dexstats.info/


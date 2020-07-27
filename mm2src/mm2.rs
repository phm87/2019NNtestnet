/******************************************************************************
 * Copyright © 2014-2019 The SuperNET Developers.                             *
 *                                                                            *
 * See the AUTHORS, DEVELOPER-AGREEMENT and LICENSE files at                  *
 * the top-level directory of this distribution for the individual copyright  *
 * holder information and the developer policies on copyright and licensing.  *
 *                                                                            *
 * Unless otherwise agreed in a custom licensing agreement, no part of the    *
 * SuperNET software, including this file may be copied, modified, propagated *
 * or distributed except according to the terms contained in the LICENSE file *
 *                                                                            *
 * Removal or modification of this copyright notice is prohibited.            *
 *                                                                            *
 ******************************************************************************/
//
//  mm2.rs
//  marketmaker
//
//  Copyright © 2017-2019 SuperNET. All rights reserved.
//

#![cfg_attr(not(feature = "native"), allow(dead_code))]
#![cfg_attr(not(feature = "native"), allow(unused_imports))]

use common::mm_ctx::MmCtxBuilder;
use common::{block_on, double_panic_crash, MM_DATETIME, MM_VERSION};

use gstuff::slurp;

use serde_json::{self as json, Value as Json};

use std::env;
use std::ffi::OsString;
use std::process::exit;
use std::ptr::null;
use std::str;

#[path = "crash_reports.rs"] pub mod crash_reports;
use self::crash_reports::init_crash_reports;

#[path = "lp_native_dex.rs"] mod lp_native_dex;
use self::lp_native_dex::{lp_init, lp_ports};

#[path = "lp_network.rs"] pub mod lp_network;

#[path = "lp_ordermatch.rs"] pub mod lp_ordermatch;
#[path = "lp_swap.rs"] pub mod lp_swap;
#[path = "rpc.rs"] pub mod rpc;

#[cfg(any(test, not(feature = "native")))]
#[path = "mm2_tests.rs"]
mod mm2_tests;

/// * `ctx_cb` - callback used to share the `MmCtx` ID with the call site.
pub fn lp_main(conf: Json, ctx_cb: &dyn Fn(u32)) -> Result<(), String> {
    if !conf["rpc_password"].is_null() {
        if !conf["rpc_password"].is_string() {
            return ERR!("rpc_password must be string");
        }

        if conf["rpc_password"].as_str() == Some("") {
            return ERR!("rpc_password must not be empty");
        }
    }

    if conf["passphrase"].is_string() {
        let netid = conf["netid"].as_u64().unwrap_or(0) as u16;
        let (_, pubport, _) = try_s!(lp_ports(netid));
        let ctx = MmCtxBuilder::new().with_conf(conf).into_mm_arc();

        if let Err(err) = ctx.init_metrics() {
            log!("Warning: couldn't initialize metricx system: "(err));
        }

        ctx_cb(try_s!(ctx.ffi_handle()));
        try_s!(block_on(lp_init(pubport, ctx)));
        Ok(())
    } else {
        ERR!("!passphrase")
    }
}

#[allow(dead_code)]
fn help() {
    // Removed options:
    // "client" - In MM2 anyone can be a Maker, the "client" option is no longer applicable.

    pintln! (
            "Command-line options.\n"
            "The first command-line argument is special and designates the mode.\n"
            "\n"
            "  help                  ..  Display this message.\n"
            "  btc2kmd {WIF or BTC}  ..  Convert a BTC WIF into a KMD WIF.\n"
            "  events                ..  Listen to a feed coming from a separate MM daemon and print it to stdout.\n"
            "  vanity {substring}    ..  Tries to find an address with the given substring.\n"
            "  nxt                   ..  Query the local NXT client (port 7876) regarding the SuperNET account in NXT.\n"
            "  {JSON configuration}  ..  Run the MarketMaker daemon.\n"
            "\n"
            "Some (but not all) of the JSON configuration parameters (* - required):\n"
            "\n"
            // We don't want to break the existing RPC API,
            // so the "refrel=coinmarketcap" designator will act as autoselect,
            // using the CoinGecko behind the scenes unless the "cmc_key" is given.
            // In the future, when MM2 is more widely used and thus we're working more tighly with the GUIs (BarterDEX, HyperDEX, dICO),
            // we might add the "refrel=cmc" and "refrel=coingecko" RPC options.
    /* we're not using the `portfolio` crate at present
            "  cmc_key        ..  CoinMarketCap Professional API Key. Switches from CoinGecko to CoinMarketCap.\n"
            "                     The Key can be obtained from 'https://pro.coinmarketcap.com/account'.\n"
    */
            "                     NB: The 'coins' command-line configuration must have the lowercased coin names in the 'name' field,\n"
          r#"                     {"coins": [{"name": "dash", "coin": "DASH", ...}, ...], ...}."# "\n"
            // cf. https://github.com/atomiclabs/hyperdex/blob/1d4ed3234b482e769124725c7e979eef5cd72d24/app/marketmaker/supported-currencies.js#L12
            "  coins          ..  Information about the currencies: their ticker symbols, names, ports, addresses, etc.\n"
            "                     If the field isn't present on the command line then we try loading it from the 'coins' file.\n"
            "  crash          ..  Simulate a crash to check how the crash handling works.\n"
            "  dbdir          ..  MM database path. 'DB' by default.\n"
            "  gui            ..  The information about GUI app using MM2 instance. Included in swap statuses shared with network.\n"
            "                 ..  It's recommended to put essential info to this field (application name, OS, version, etc).\n"
            "                 ..  e.g. AtomicDEX iOS 1.0.1000.\n"
            "  myipaddr       ..  IP address to bind to for P2P networking.\n"
            "  netid          ..  Subnetwork. Affects ports and keys.\n"
            "  passphrase *   ..  Wallet seed.\n"
            "                     Compressed WIFs and hexadecimal ECDSA keys (prefixed with 0x) are also accepted.\n"
            "  panic          ..  Simulate a panic to see if backtrace works.\n"
            // cf. https://github.com/atomiclabs/hyperdex/pull/563/commits/6d17c0c994693b768e30130855c679a7849a2b27
            "  rpccors        ..  Access-Control-Allow-Origin header value to be used in all the RPC responses.\n"
            "                     Default is currently 'http://localhost:3000'\n"
            "  rpcip          ..  IP address to bind to for RPC server. Overrides the 127.0.0.1 default\n"
            "  rpc_password   ..  RPC password used to authorize non-public RPC calls\n"
            "                     MM generates password from passphrase if this field is not set\n"
            "  rpc_local_only ..  MM forbids some RPC requests from not loopback (localhost) IPs as additional security measure.\n"
            "                     Defaults to `true`, set `false` to disable. `Use with caution`.\n"
            "  rpcport        ..  If > 1000 overrides the 7783 default.\n"
            "  i_am_seed      ..  Activate the seed node mode (acting as a relay for mm2 clients).\n"
            "                     Defaults to `false`.\n"
            "  seednodes      ..  Seednode IPs that node will use.\n"
            "                     At least one seed IP must be present if the node is not a seed itself.\n"
            "  stderr         ..  Print a message to stderr and exit.\n"
            "  userhome       ..  System home directory of a user ('/root' by default).\n"
            "  wif            ..  `1` to add WIFs to the information we provide about a coin.\n"
            "\n"
            "Environment variables:\n"
            "\n"
            // Per-process (we might have several MM instances running but they will share the same log).
            "  MM_CONF_PATH   ..  File path. MM2 will try to load the JSON configuration from this file.\n"
            "                     File must contain valid json with structure mentioned above.\n"
            "                     Defaults to `MM2.json`\n"
            "  MM_COINS_PATH  ..  File path. MM2 will try to load coins data from this file.\n"
            "                     File must contain valid json.\n"
            "                     Recommended: https://github.com/jl777/coins/blob/master/coins.\n"
            "                     Defaults to `coins`.\n"
            "  MM_LOG         ..  File path. Must end with '.log'. MM will log to this file.\n"
            "\n"
            // Generated from https://github.com/KomodoPlatform/developer-docs/tree/sidd.
            // (SHossain, siddhartha-crypto).
            "See also the online documentation at\n"
            "https://developers.atomicdex.io\n"
        )
}

#[cfg(feature = "native")]
#[allow(dead_code)] // Not used by mm2_lib.
pub fn mm2_main() {
    use libc::c_char;

    init_crash_reports();
    log!({"AtomicDEX MarketMaker {} DT {}", MM_VERSION, MM_DATETIME});

    // Temporarily simulate `argv[]` for the C version of the main method.
    let args: Vec<String> = env::args()
        .map(|mut arg| {
            arg.push('\0');
            arg
        })
        .collect();
    let mut args: Vec<*const c_char> = args.iter().map(|s| s.as_ptr() as *const c_char).collect();
    args.push(null());

    let args_os: Vec<OsString> = env::args_os().collect();

    // NB: The first argument is special, being used as the mode switcher.
    // The other arguments might be used to pass the data to the various MM modes,
    // we're not checking them for the mode switches in order not to risk [untrusted] data being mistaken for a mode switch.
    let first_arg = args_os.get(1).and_then(|arg| arg.to_str());

    if first_arg == Some("panic") {
        panic!("panic message")
    }
    if first_arg == Some("crash") {
        double_panic_crash()
    }
    if first_arg == Some("stderr") {
        eprintln!("This goes to stderr");
        return;
    }

    if first_arg == Some("--help") || first_arg == Some("-h") || first_arg == Some("help") {
        help();
        return;
    }
    if cfg!(windows) && first_arg == Some("/?") {
        help();
        return;
    }

    if let Err(err) = run_lp_main(first_arg, &|_| ()) {
        log!((err));
        exit(1);
    }
}

// LND
extern crate protoc_grpcio;  

/// Parses the `first_arg` as JSON and runs LP_main.
/// Attempts to load the config from `MM2.json` file if `first_arg` is None
///
/// * `ctx_cb` - Invoked with the MM context handle,
///              allowing the `run_lp_main` caller to communicate with MM.
pub fn run_lp_main(first_arg: Option<&str>, ctx_cb: &dyn Fn(u32)) -> Result<(), String> {
    let conf_path = env::var("MM_CONF_PATH").unwrap_or_else(|_| "MM2.json".into());
    let conf_from_file = slurp(&conf_path);
    let conf = match first_arg {
        Some(s) => s,
        None => {
            if conf_from_file.is_empty() {
                return ERR!(
                    "Config is not set from command line arg and {} file doesn't exist.",
                    conf_path
                );
            }
            try_s!(std::str::from_utf8(&conf_from_file))
        },
    };

    let mut conf: Json = match json::from_str(conf) {
        Ok(json) => json,
        Err(err) => return ERR!("couldnt parse.({}).{}", conf, err),
    };

    if conf["coins"].is_null() {
        let coins_path = env::var("MM_COINS_PATH").unwrap_or_else(|_| "coins".into());
        let coins_from_file = slurp(&coins_path);
        if coins_from_file.is_empty() {
            return ERR!(
                "No coins are set in JSON config and '{}' file doesn't exist",
                coins_path
            );
        }
        conf["coins"] = match json::from_slice(&coins_from_file) {
            Ok(j) => j,
            Err(e) => {
                return ERR!(
                    "Error {} parsing the coins file, please ensure it contains valid json",
                    e
                )
            },
        }
    }

    let proto_file = "rpc.proto";
    let proto_root = "src/proto";  
    let rust_lnd = "src/lnd";    
    println!("cargo:rerun-if-changed={}", proto_root);  
    protoc_grpcio::compile_grpc_protos(
          &[proto_file],
          &[proto_root],
          &rust_lnd,  
    ).expect("Failed to compile gRPC definitions!");  
    
    try_s!(lp_main(conf, ctx_cb));
    Ok(())
}

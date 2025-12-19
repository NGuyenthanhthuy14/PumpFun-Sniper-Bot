use crate::*;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    commitment_config::CommitmentLevel,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::Duration;

use crate::CONFIG;

//Bot mode
pub static DEV_MODE: Lazy<bool> = Lazy::new(|| CONFIG.mode.is_dev_mode);
pub static BUY_TX_COUNTER: Lazy<AtomicI32> =
    Lazy::new(|| AtomicI32::new(CONFIG.mode.buy_tx_counter));

pub fn decrese_buy_tx_remain_counter() {
    BUY_TX_COUNTER.fetch_sub(1, Ordering::SeqCst);
}

pub fn get_buy_tx_remain_counter() -> i32 {
    BUY_TX_COUNTER.load(Ordering::SeqCst)
}

//Wallet key
pub static SIGNER_KEYPAIR: Lazy<Keypair> = Lazy::new(|| {
    let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
    wallet
});

pub static SIGNER_PUBKEY: Lazy<Pubkey> = Lazy::new(|| {
    let wallet: Keypair = Keypair::from_base58_string(&CONFIG.wallet_config.private_key);
    wallet.pubkey()
});

//Target wallets
pub static TARGET_WALLETS: Lazy<Vec<String>> =
    Lazy::new(|| CONFIG.target_config.target_wallets.clone());
//HTTP endpoint
pub static ZERO_SLOT_HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    println!("🔄 Initializing 0-slot HTTP client...");
    
    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(300))     // 5 minutes
        .pool_max_idle_per_host(5)                       // Multiple connections
        .tcp_keepalive(Duration::from_secs(10))          // Frequent keep-alive
        .tcp_nodelay(true)                               // MUST for low latency
        .connect_timeout(Duration::from_secs(3))         // Fast connection
        .timeout(Duration::from_secs(10))                // Reasonable timeout
        .http2_keep_alive_interval(Duration::from_secs(20))
        .http2_keep_alive_timeout(Duration::from_secs(90))
        .http2_keep_alive_while_idle(true)
        .use_rustls_tls()                                // Faster TLS
        .build()
        .expect("Failed to build 0-slot HTTP client");
    
    // PRE-WARM THIS SPECIFIC ENDPOINT
    let client_arc = Arc::new(client);
    pre_warm_zero_slot_endpoint(client_arc.clone());
    
    client_arc
});

//RPC endpoint
pub static RPC_ENDPOINT: Lazy<String> = Lazy::new(|| CONFIG.connection_config.rpc_endpoint.clone());
pub static RPC_CLIENT: Lazy<Arc<RpcClient>> = Lazy::new(|| {
    Arc::new(RpcClient::new_with_commitment(
        CONFIG.connection_config.rpc_endpoint.clone(),
        CommitmentConfig {
            commitment: CommitmentLevel::Processed,
        },
    ))
});
pub static GRPC_ENDPOINT: Lazy<String> =
    Lazy::new(|| CONFIG.connection_config.grpc_endpoint.clone());
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(|| CONFIG.connection_config.grpc_token.clone());

//Confirm service
pub static CONFIRM_SERVICE: Lazy<String> =
    Lazy::new(|| CONFIG.relayer_config.confirm_service.clone());

//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.buy_amount_sol);

//Buy Condition
pub static BUY_CONDITION_PRICE_FALL_PCNT: Lazy<f64> =
    Lazy::new(|| CONFIG.buy_condition_config.price_variant_width_percent / 100.0);

//Slippage
pub static SLIPPAGE: Lazy<f64> =
    Lazy::new(|| 1.0 + CONFIG.slippage_config.slippage_percent as f64 / 100.0);

//Fee
pub static PRIORITY_FEE: Lazy<(u64, u64, f64)> = Lazy::new(|| {
    let cu: u64 = CONFIG.fee_config.cu;
    let priority_fee_micro_lamport = CONFIG.fee_config.priority_fee_micro_lamport;

    let third_party_fee = CONFIG.fee_config.third_party_fee;

    (cu, priority_fee_micro_lamport, third_party_fee)
});

//Filter
pub static RUG_DETECT: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.rug_detect);
pub static BUNDLE_TX_LIMIT: Lazy<i32> = Lazy::new(|| CONFIG.filter_setting.bundle_tx_limit);

pub static VOLUME_FILTER: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.volume_filter);
pub static MIN_VOLUME_LIMIT_SOL: Lazy<i32> =
    Lazy::new(|| CONFIG.filter_setting.min_volume_limit_sol);

pub static MARKET_CAP_FILTER: Lazy<bool> = Lazy::new(|| CONFIG.filter_setting.market_cap_filter);
pub static MIN_MARKET_CAP_LIMIT_SOL: Lazy<i32> =
    Lazy::new(|| CONFIG.filter_setting.min_market_cap_limit_sol);

pub static MAX_TOKEN_HOLDER_FILTER: Lazy<bool> =
    Lazy::new(|| CONFIG.filter_setting.max_token_holder_filter);
pub static MAX_TOKEN_HOLDER_LIMIT: Lazy<u64> =
    Lazy::new(|| CONFIG.filter_setting.max_token_holder_limit);

//Stop monitor
pub static STOP_NO_ACTIVITY_TOKEN_MONITORING: Lazy<bool> =
    Lazy::new(|| CONFIG.monitor_setting.stop_no_activity_token_monitoring);
pub static NO_ACTIVITY_TIME: Lazy<u64> = Lazy::new(|| CONFIG.monitor_setting.no_activity_time);

lazy_static! {
    pub static ref AUTO_TURNOFF: AtomicBool = AtomicBool::new(false);
}

pub async fn show_bot_settings() {
    log!("Public key: {:?}", *SIGNER_PUBKEY);
    log!("Confirm service: {:?}", *CONFIRM_SERVICE);
    log!("Buy settings: {:?}", CONFIG.buy_setting);
    log!("Slippage: {:?}%", *SLIPPAGE);
    log!("Grpc endpoint: {:?}", *GRPC_ENDPOINT);
    log!("Grpc token: {:?}", *GRPC_TOKEN);
    log!("RPC endpoint: {:?}", *RPC_ENDPOINT);
    log!("Rug detect: {:?}", *RUG_DETECT);
    log!("Bundle tx limit: {:?}", *BUNDLE_TX_LIMIT);
    log!("Volume filter: {:?}", *VOLUME_FILTER);
    log!("Min volume limit: {:?} SOL", *MIN_VOLUME_LIMIT_SOL);
    log!("Marketcap filter: {:?}", *MARKET_CAP_FILTER);
    log!("Min marketcap limit: {:?} SOL", *MIN_MARKET_CAP_LIMIT_SOL);
    log!(
        "Stop no activity token monitoring: {:?}",
        *STOP_NO_ACTIVITY_TOKEN_MONITORING
    );
    log!("No activity time: {:?} seconds", *NO_ACTIVITY_TIME);

    init_validator();
    // connect_timer_service().await;

    log!(
        "TAKE_PROFIT_1 : {:<5.3} % , TAKE_PROFIT_2 : {:<5.3} % , TAKE_PROFIT_3 : {:<5.3} % , TAKE_PROFIT_4 : {:<5.3} % , TAKE_PROFIT_5 : {:<5.3} % , SL_1 : {:<5.3} %, SL_2 : {:<5.3} %, SL_3 : {:<5.3} %",
        *TAKE_PROFIT_1 * 100.0,
        *TAKE_PROFIT_2 * 100.0,
        *TAKE_PROFIT_3 * 100.0,
        *TAKE_PROFIT_4 * 100.0,
        *TAKE_PROFIT_5 * 100.0,
        *STOP_LOSS_1 * 100.0,
        *STOP_LOSS_2 * 100.0,
        *STOP_LOSS_3 * 100.0
    );
    log!(
        "TS_1 : {:<5.3} %, TS_1_STOP : {:<5.3} %, TS_1_SELL_PCNT : {:<5.3} %",
        *TS_1 * 100.0,
        *TS_1 * (1.0 - *TS_1_STOP) * 100.0,
        *TS_1_SELL_PCNT * 100.0
    );
    log!(
        "TS_2 : {:<5.3} %, TS_2_STOP : {:<5.3} %, TS_2_SELL_PCNT : {:<5.3} %",
        *TS_2 * 100.0,
        *TS_2 * (1.0 - *TS_2_STOP) * 100.0,
        *TS_2_SELL_PCNT * 100.0
    );
    log!(
        "TS_3 : {:<5.3} %, TS_3_STOP : {:<5.3} %, TS_3_SELL_PCNT : {:<5.3} %",
        *TS_3 * 100.0,
        *TS_3 * (1.0 - *TS_3_STOP) * 100.0,
        *TS_3_SELL_PCNT * 100.0
    );
    log!(
        "TS_4 : {:<5.3} %, TS_4_STOP : {:<5.3} %, TS_4_SELL_PCNT : {:<5.3} %",
        *TS_4 * 100.0,
        *TS_4 * (1.0 - *TS_4_STOP) * 100.0,
        *TS_4_SELL_PCNT * 100.0
    );
    log!(
        "TS_5 : {:<5.3} %, TS_5_STOP : {:<5.3} %, TS_5_SELL_PCNT : {:<5.3} %",
        *TS_5 * 100.0,
        *TS_5 * (1.0 - *TS_5_STOP) * 100.0,
        *TS_5_SELL_PCNT * 100.0
    );
}

pub fn pre_warm_zero_slot_endpoint(client: Arc<Client>) {
    tokio::spawn(async move {
        println!("🔥 Pre-warming 0-slot endpoint...");
        
        // Try multiple times to establish connection
        for attempt in 1..=3 {
            let url = "http://la1.0slot.trade?api-key=335e371309b6492584368e9dc553622d".to_string();
            
            match client.get(&url).send().await {
                Ok(response) => {
                    println!("✅ 0-slot endpoint ready (attempt {}): HTTP {}", 
                        attempt, response.status());
                    
                    // If it's a 404 or similar, that's OK - connection is established
                    if response.status().is_success() {
                        println!("🎯 Successfully connected to 0-slot service");
                    }
                    break;
                }
                Err(e) if attempt < 3 => {
                    println!("⚠️ 0-slot warm-up attempt {} failed: {:?}", attempt, e);
                    tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
                }
                Err(e) => {
                    eprintln!("❌ Failed to pre-warm 0-slot endpoint: {:?}", e);
                }
            }
        }
    });
}

pub fn get_zero_slot_client() -> Arc<Client> {
    // This forces initialization on first call
    ZERO_SLOT_HTTP_CLIENT.clone()
}

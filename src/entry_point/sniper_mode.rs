use pumpfun_sniper::*;
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

const PATTERN_SERVER_PORT: u16 = 3355;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    info!("{}", SNIPER_MODE_STR.green());
    let client = get_http_client();

    match LANDING_SERVICE.as_str() {
        "ZERO_SLOT" => {
            pre_warm_zero_slot_endpoint(client).await;
        }
        "HELIUS" => {
            pre_warm_helius_endpoint(client).await;
        }
        _ => {
            println!("Unsupported landing service, defaulting to 0-slot");
            pre_warm_zero_slot_endpoint(client).await;
        }
    }

    init_nonce_pool().await;

    // Spawn dead-token no-activity checker
    if *STOP_NO_ACTIVITY_TOKEN_MONITORING {
        info!(
            "No-activity monitoring: ENABLED (threshold: {} seconds)",
            *NO_ACTIVITY_TIME
        );
        tokio::spawn(async {
            loop {
                check_no_activity_tokens().await;
            }
        });
    }

    // Load manual patterns at startup
    let manual_count = get_manual_patterns().len();
    info!("Loaded {} manual pattern(s)", manual_count);

    // Show dynamic buy amount mode
    if *DYNAMIC_BUY_AMOUNT_MODE {
        info!(
            "Dynamic Buy Amount: ENABLED ({}L → {:.2}x, {}W → {:.2}x, max {:.2}x, min {:.2}x)",
            *LOSS_SEQUENCE, *LOSS_MULTIPLY,
            *PROFIT_SEQUENCE, *PROFIT_MULTIPLY,
            *MAX_BUY_AMOUNT_MULTIPLY, *MIN_BUY_AMOUNT_MULTIPLY,
        );
    }

    tokio::spawn(async {
        run_pattern_server(PATTERN_SERVER_PORT).await;
    });

    // ══════════════════════════════════════════════════════════════════
    // Phase 2: Anti-Rug Intelligence Layer — Initialization
    // ══════════════════════════════════════════════════════════════════
    info!("══════════════════════════════════════════════════");
    info!("  Phase 2 — Anti-Rug Intelligence Layer");
    info!("  ────────────────────────────────────────────");
    info!("  Genesis Detector:   {}", if *GENESIS_FILTER_ENABLED { "ENABLED" } else { "DISABLED" });
    if *GENESIS_FILTER_ENABLED {
        info!("    Max genesis buy:  {:.0}%", *MAX_GENESIS_BUY_PERCENT);
        info!("    Max single whale: {:.0}%", *MAX_SINGLE_WALLET_PERCENT);
        info!("    Slot window:      {} slots", *GENESIS_SLOT_WINDOW);
        info!("    Max clusters:     {} wallets", *MAX_CLUSTERED_WALLETS);
    }
    info!("  Wallet Profiler:    {}", if *WALLET_PROFILER_ENABLED { "ENABLED" } else { "DISABLED" });
    if *WALLET_PROFILER_ENABLED {
        info!("    Min wallet age:   {}h", *MIN_WALLET_AGE_HOURS);
        info!("    Min TX history:   {}", *MIN_HISTORICAL_TX_COUNT);
        info!("    Block CEX funded: {}", *BLOCK_CEX_FUNDED);
        info!("    RPC timeout:      {}ms", *WALLET_RPC_TIMEOUT_MS);
        info!("    Known CEX addrs:  {}", total_known_cex_wallets());
    }
    info!("  Metadata Checker:   {}", if *METADATA_CHECKER_ENABLED { "ENABLED" } else { "DISABLED" });
    if *METADATA_CHECKER_ENABLED {
        info!("    Require URI:      {}", *REQUIRE_METADATA_URI);
        info!("    Fetch URI JSON:   {}", *FETCH_URI_CONTENT);
        info!("    Empty action:     {}", *METADATA_EMPTY_ACTION);
    }
    info!("  ────────────────────────────────────────────");
    info!("  Max Risk Score:     {:.0}", *MAX_TOTAL_RISK_SCORE);
    info!("  Dynamic Sizing:     {}", if *ENABLE_DYNAMIC_SIZING { "ENABLED" } else { "DISABLED" });
    info!("  Filter CSV Log:     {}", if *FILTER_LOG_ENABLED { &*FILTER_LOG_DIR } else { "DISABLED" });
    info!("  Telegram Notify:    {}", if tg_notify_enabled() { "ENABLED" } else { "DISABLED (set TG_BOT_TOKEN + TG_CHAT_ID)" });
    info!("══════════════════════════════════════════════════");

    // Spawn Phase 2 cleanup tasks (every 30s)
    tokio::spawn(async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            genesis_cleanup();
            wallet_profile_cache_cleanup();
            metadata_name_tracker_cleanup();
        }
    });

    // Spawn Telegram Control Bot (if configured)
    tokio::spawn(async {
        crate::modules::pre_buy_filter::tg_control::start_telegram_control_bot().await;
    });

    let grpc_config = GrpcClientConfig::new(
        "sniper_mode".to_string(),
        GRPC_ENDPOINT.to_string(),
        GRPC_TOKEN.to_string(),
    );

    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![
            PUMPFUN_PROGRAM_ID.to_string(),
            PUMPSWAP_PROGRAM_ID.to_string(),
        ],
        account_exclude: vec![],
        account_required: vec![],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    };

    if let Err(e) = grpc_config
        .subscribe_with_reconnect(subscribe_pumpfun_program_id)
        .await
    {
        error!(
            "Failed to maintain GRPC connection after all retries: {:?}",
            e
        );
    }
}

use colored::*;
use pumpfun_sniper::*;
use std::sync::atomic::Ordering;
use tokio::time::{Duration, interval};
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

#[tokio::main]
pub async fn main() {
    info!("{}", SNIPER_MODE_STR.green());
    show_bot_settings().await;

    tokio::spawn({
        async {
            loop {
                recent_blockhash_handler().await;
            }
        }
    });

    let mut interval = interval(Duration::from_millis(30000));

    tokio::spawn({
        async move {
            loop {
                interval.tick().await;
                let start_selling = check_auto_turn_off_time("monitor_mode");
                if start_selling {
                    AUTO_TURNOFF.store(true, Ordering::Relaxed);
                };
            }
        }
    });

    let grpc_config = GrpcClientConfig::new(
        "monitor_mode".to_string(),
        GRPC_ENDPOINT.to_string(),
        GRPC_TOKEN.to_string(),
    );

    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![PUMPFUN_PROGRAM_ID.to_string()],
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

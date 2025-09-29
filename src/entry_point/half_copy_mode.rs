use pumpfun_sniper::*;
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;
use colored::*;

#[tokio::main]
pub async fn main() {
    info!("{}", HALF_COPY_MODE_STR.green());
    show_bot_settings();

    init_jito().await;
    init_nozomi().await;
    init_zero_slot().await;

    load_blacklist();

    tokio::spawn({
        async {
            loop {
                recent_blockhash_handler().await;
            }
        }
    });

    let mut grpc_client = setup_grpc_client(GRPC_ENDPOINT.to_string(), GRPC_TOKEN.to_string())
        .await
        .unwrap();

    let (subscribe_tx, subscribe_rx) = grpc_client.subscribe().await.unwrap();
    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![],
        account_exclude: vec![],
        account_required: vec![PUMPFUN_PROGRAM_ID.to_string()],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    };

    send_subscription_request_grpc(subscribe_tx, subscribe_pumpfun_program_id)
        .await
        .unwrap();

      let _ = process_half_copy_mode(subscribe_rx).await;
}

use crate::*;
use futures::StreamExt;
use std::sync::atomic::Ordering;
use yellowstone_grpc_proto::{geyser::SubscribeUpdate, tonic::Status};

pub async fn process_monitor_mode<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                if AUTO_TURNOFF.load(Ordering::Relaxed) {
                    break;
                };
                let (account_keys, ixs, inner_ixs, tx_id, _signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };
                let ix_info =
                    filter_by_program_id(ixs.clone(), inner_ixs.clone(), account_keys.clone(), PUMPFUN_PROGRAM_ID)
                        .unwrap();
                let budget_compute_ix_info = filter_by_program_id(ixs, inner_ixs, account_keys.clone(), BUDGET_COMPUTE_PROGRAM).unwrap();


                let trade_data = get_trade_info(ix_info, account_keys.clone());
                let budget_compute_data = get_budget_compute_info(budget_compute_ix_info);

                let _ = handle_sniper_event(trade_data, budget_compute_data, tx_id).await;
            }

            Err(e) => {
                log!("Stream error: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}

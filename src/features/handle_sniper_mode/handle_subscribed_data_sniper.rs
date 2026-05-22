use crate::*;
use futures::StreamExt;
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdate, subscribe_update::UpdateOneof},
    tonic::Status,
};

pub async fn process_sniper_mode<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: StreamExt<Item = Result<SubscribeUpdate, Status>> + Unpin,
{
    while let Some(result) = stream.next().await {
        match result {
            Ok(update) => {
                let (account_keys, ixs, inner_ixs, tx_id, _signers) =
                    if let Some(data) = extract_transaction_data(&update) {
                        data
                    } else {
                        continue;
                    };

                let mut grouped = group_by_program_ids(
                    ixs,
                    inner_ixs,
                    &[BUDGET_COMPUTE_PROGRAM, PUMPFUN_PROGRAM_ID, PUMPSWAP_PROGRAM_ID],
                    &account_keys,
                );
                // Order matches the program_ids slice above
                let ix_info_pumpswap = grouped.pop().unwrap();
                let ix_info_pumpfun = grouped.pop().unwrap();
                let budget_compute_ix_info = grouped.pop().unwrap();

                let mut all_pump_ix = Vec::with_capacity(ix_info_pumpfun.len() + ix_info_pumpswap.len());
                all_pump_ix.extend(ix_info_pumpfun.clone());
                all_pump_ix.extend(ix_info_pumpswap.clone());

                let transaction_update = match &update.update_oneof {
                    Some(UpdateOneof::Transaction(tx_update)) => {
                        match tx_update.transaction.as_ref() {
                            Some(tx) => tx,
                            None => continue,
                        }
                    }
                    _ => continue,
                };

                // Phase 2: Extract slot from gRPC update for genesis tracking
                let tx_slot = match &update.update_oneof {
                    Some(UpdateOneof::Transaction(tx_update)) => tx_update.slot,
                    _ => 0,
                };

                let budget_compute_data = get_budget_compute_info(budget_compute_ix_info);
                let pumpfun_trade_data =
                    get_pumpfun_trade_info(ix_info_pumpfun.clone(), account_keys.clone(), transaction_update);

                let migration_data = migrate_info(all_pump_ix.clone(), account_keys.clone());

                let pumpswap_trade_data =
                    get_pumpswap_trade_info(ix_info_pumpswap.clone(), account_keys.clone());

                let trade_token_data_map = handle_trade_events(
                    budget_compute_data,
                    pumpfun_trade_data,
                    migration_data,
                    pumpswap_trade_data,
                    tx_id.clone(),
                    tx_slot,  // Phase 2: pass slot for genesis tracking
                )
                .await;

                make_sniper_tx(&trade_token_data_map).await;
            }

            Err(e) => {
                log!("Stream error: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}

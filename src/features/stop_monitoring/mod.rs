use crate::*;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use tokio::time::{Duration, sleep};

pub async fn check_no_activity_tokens() {
    if *STOP_NO_ACTIVITY_TOKEN_MONITORING {
        let keys: Vec<Pubkey> = TOKEN_DB
            .map
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        for token_key in keys {
            if let Some(mut token_data) = TOKEN_DB.get(token_key).ok().flatten() {
                if token_data.token_last_activity_time.elapsed()
                    >= Duration::from_secs(*NO_ACTIVITY_TIME)
                {
                    let instruction: (Vec<Instruction>, String, u64) = if token_data.token_is_purchased {
                        if token_data.token_sell_status != TokenSellStatus::SellTradeSubmitted {
                            token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                            let _ = TOKEN_DB.upsert(token_key, token_data.clone());

                            let tag = format!(
                                "[Sell]\t*Stop monitoring\t\t*Mint: {}\t*No activity in last {} seconds",
                                token_key, *NO_ACTIVITY_TIME
                            );
                            alert!(
                                "[Sell]\t*Stop monitoring\t\t*Mint: {}\t*No activity in last {} seconds",
                                token_key,
                                *NO_ACTIVITY_TIME
                            );

                            let mut ix: Vec<Instruction> = Vec::new();
                            if token_data.token_is_migrated {
                                if let Some(mut pumpswap_struct) = token_data.pumpswap_struct {
                                    let create_ata_ix = pumpswap_struct.get_create_ata_idempotent_ix();
                                    let sell_ix = pumpswap_struct.get_sell_ix(
                                        token_data.token_balance,
                                        token_data.token_creator,
                                        token_data.is_cashback_enabled,
                                    );
                                    let close_ix = pumpswap_struct.close_wsol_ata();
                                    ix.extend(create_ata_ix);
                                    ix.push(sell_ix);
                                    ix.push(close_ix);
                                }
                            } else {
                                let sell_ix = token_data.pumpfun_struct.get_sell_ix(
                                    token_data.token_creator,
                                    token_data.token_balance,
                                    token_data.is_cashback_enabled,
                                );
                                let close_ata_ix = token_data.pumpfun_struct.get_close_ata_ix();
                                ix.push(sell_ix);
                                ix.push(close_ata_ix);
                            }

                            (ix, tag, token_data.token_balance)
                        } else {
                            (vec![], "".to_string(), 0)
                        }
                    } else {
                        // Token was tracked but not purchased (BUY may have failed)
                        // Release position slot and clean up
                        alert!(
                            "[Stop-Tracking]\t\t*Mint: {}\t*No activity in last {} seconds",
                            token_key,
                            *NO_ACTIVITY_TIME
                        );
                        decrement_open_positions();
                        info!(
                            "📊 [STOP_TRACK_RELEASED] Position released | Positions: {}/{} | MINT: {}",
                            get_open_positions(), *MAX_OPEN_POSITIONS, token_key
                        );
                        let _ = TOKEN_DB.delete(token_key);

                        (vec![], "".to_string(), 0)
                    };

                    let (ix, tag, sell_amount) = instruction;

                    if !ix.is_empty() {
                        let mint = token_key;
                        tokio::spawn(async move {
                            let _ = confirm_sell_with_retry(mint, sell_amount, ix, tag).await;
                        });
                    }
                }
            }
        }
    }

    sleep(Duration::from_millis(1000)).await;
}

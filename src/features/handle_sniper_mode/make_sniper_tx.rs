use crate::*;
use dashmap::DashMap;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub async fn make_sniper_tx(trade_token_data_map: &DashMap<Pubkey, TokenDatabaseSchema>) {
    for trade_token_data in trade_token_data_map.iter() {
        let mut token_data = trade_token_data.value().clone();
        let instructions: (Vec<Instruction>, String) = if token_data.token_trade_signal
            == TokenTradeSignal::IsEntryPoint
        {
            let buy_tx_remaining_counter = get_buy_tx_remain_counter();

            if !*DEV_MODE || buy_tx_remaining_counter != 0 {
                decrese_buy_tx_remain_counter();

                let mut ix: Vec<Instruction> = Vec::new();
                let create_ata_ix = token_data
                    .pump_fun_swap_accounts
                    .get_create_ata_idempotent_ix();
                let buy_ix = token_data
                    .pump_fun_swap_accounts
                    .get_buy_ix(*BUY_AMOUNT_SOL * 10f64.powi(9), token_data.token_price);

                ix.push(create_ata_ix);
                ix.push(buy_ix);

                token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                token_data.token_trade_signal = TokenTradeSignal::EntrySubmitted;

                let tag = format!("Token price is lower than initial_bundle_price x0.8, buying...");

                let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

                (ix, tag)
            } else {
                (vec![], "".to_string())
            }
        } else if token_data.tp_state == TPMode::TP4 && token_data.tracked_tp_state != TPMode::TP4 {
            let sell_amount = if token_data.token_balance > token_data.tp_selling_plan.tp_4 {
                token_data.tp_selling_plan.tp_4
            } else {
                token_data.token_balance
            };
            let sell_ix: Instruction = token_data.pump_fun_swap_accounts.get_sell_ix(sell_amount);
            // let close_ata = token_data.pump_fun_swap_accounts.get_close_ata_ix();

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);
            // ix.push(close_ata);

            token_data.tracked_tp_state = TPMode::TP4;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP4 triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint, token_data.token_marketcap, sell_amount,
            );

            info!(
                "[SELL]\t*TP4 triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint, token_data.token_marketcap, sell_amount,
            );

            (ix, tag)
        } else if token_data.tp_state == TPMode::TP5 && token_data.tracked_tp_state != TPMode::TP5 {
            let sell_amount = if token_data.token_balance > token_data.tp_selling_plan.tp_5 {
                token_data.tp_selling_plan.tp_5
            } else {
                token_data.token_balance
            };
            let sell_ix: Instruction = token_data.pump_fun_swap_accounts.get_sell_ix(sell_amount);
            let close_ata = token_data.pump_fun_swap_accounts.get_close_ata_ix();

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);
            ix.push(close_ata);

            token_data.tracked_tp_state = TPMode::TP5;
            let _ = TOKEN_DB.upsert(token_data.token_mint, token_data.clone());

            let tag = format!(
                "[SELL]\t*TP5 triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint, token_data.token_marketcap, sell_amount,
            );

            info!(
                "[SELL]\t*TP5 triggered\t*MINT: {}\t*MC: {}\t*AMOUNT: {}",
                token_data.pump_fun_swap_accounts.mint, token_data.token_marketcap, sell_amount,
            );

            (ix, tag)
        } else {
            (vec![], "".to_string())
        };

        let (ix, tag) = instructions;

        if !ix.is_empty() {
            tokio::spawn(async move {
                let _ = confirm(ix, tag).await;
            });
        }
    }
}

use crate::*;
use dashmap::DashMap;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn handle_sniper(trade_token_data_map: &DashMap<Pubkey, TokenDatabaseSchema>) {
    for trade_token_data in trade_token_data_map.iter() {
        let mut token_data = trade_token_data.value().clone();
        let instructions: (Vec<Instruction>, String) = if !token_data.token_is_purchased && token_data.token_event == TokenEventType::MintTokenEvent {
            let mut ix: Vec<Instruction> = Vec::new();
            let create_ata_ix = token_data
                .pump_fun_swap_accounts
                .get_create_ata_idempotent_ix();
            let transfer_sol_ix = token_data.pump_fun_swap_accounts.get_sol_ix();
            let buy_ix = token_data
                .pump_fun_swap_accounts
                .get_buy_ix(token_data.token_price);

            ix.push(create_ata_ix);
            ix.push(transfer_sol_ix);
            ix.push(buy_ix);
            let tag = format!(
                "[Buy]
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {} SOL",
                token_data.pump_fun_swap_accounts.mint, token_data.token_price, *BUY_AMOUNT_SOL
            );

            info!(
                "[Buy]
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {} SOL",
                token_data.pump_fun_swap_accounts.mint, token_data.token_price, *BUY_AMOUNT_SOL
            );
            (ix, tag)
        } else if token_data.ts_states == TSMode::TS5Stop {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_5_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TS_5_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_5_stop
            );

            info!(
                "[SELL]
                    \t* TS_5_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_5_stop
            );

            (ix, tag)
        } else if token_data.ts_states == TSMode::TS4Stop {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_4_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TS_4_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_4_stop,
            );

            info!(
                "[SELL]
                    \t* TS_4_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_4_stop,
            );

            (ix, tag)
        } else if token_data.ts_states == TSMode::TS3Stop {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_3_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TS_3_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_3_stop,
            );

            info!(
                "[SELL]
                    \t* TS_3_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_3_stop,
            );

            (ix, tag)
        } else if token_data.ts_states == TSMode::TS2Stop {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_2_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TS_2_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_2_stop,
            );

            info!(
                "[SELL]
                    \t* TS_2_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_2_stop,
            );

            (ix, tag)
        } else if token_data.ts_states == TSMode::TS1Stop {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.ts_stop_selling_plan.ts_1_stop);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TS_1_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_1_stop,
            );

            info!(
                "[SELL]
                    \t* TS_1_Stop
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.ts_stop_selling_plan.ts_1_stop,
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::TP1 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_1);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TP1
                    \t* MINT : {}
                    \t* PRICE : {}
                    \t* AMOUNT : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_1,
            );

            info!(
                "[SELL]
                    \t* TP1
                    \t* MINT : {}
                    \t* PRICE : {}
                    \t* AMOUNT : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_1,
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::TP2 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_2);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_2,
            );

            info!(
                "[SELL]
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_2,
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::TP3 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_3);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TP3
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_3,
            );

            info!(
                "[SELL]
                    \t* TP3
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_3,
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::TP4 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_4);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TP4
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_4,
            );

            info!(
                "[SELL]
                    \t* TP4
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_4
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::TP5 {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.tp_selling_plan.tp_5);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* TP5
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_5,
            );

            info!(
                "[SELL]
                    \t* TP5
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.tp_selling_plan.tp_5
            );

            (ix, tag)
        } else if token_data.tp_states == TPMode::SL {
            let sell_ix: Instruction = token_data
                .pump_fun_swap_accounts
                .get_sell_ix(token_data.token_balance);

            let mut ix: Vec<Instruction> = Vec::new();
            ix.push(sell_ix);

            let tag = format!(
                "[SELL]
                    \t* STOP_LOSS
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance,
            );

            info!(
                "[SELL]
                    \t* STOP_LOSS
                    \t* Mint : {}
                    \t* Price : {}
                    \t* Amount : {}",
                token_data.pump_fun_swap_accounts.mint,
                token_data.token_price,
                token_data.token_balance,
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

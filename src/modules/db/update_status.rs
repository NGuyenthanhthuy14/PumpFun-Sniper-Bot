use crate::*;
use colored::*;
use std::time::Duration;

pub fn update_status_from_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: BuyEvent,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    info!(
        "[Token Buy]\t*Mint: {}\t*User: {}\t*MC: {:.2}*Amount: {:?}\t*Holders: {}",
        token_data.token_mint,
        buy_event.user,
        token_data.token_marketcap,
        buy_event.sol_amount as f64 / 10f64.powi(9),
        token_data.token_holders.len()
    );

    if token_data.token_mint_time.elapsed() <= Duration::from_millis(50)
        && buy_event.user != token_data.token_creator
    {
        // warning!("[SNIPER]\t");
        token_data.sniper_buy_amount += buy_event.sol_amount;
    }

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + buy_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    if buy_event.user == token_data.token_creator {
        dev_trade!(
            "Dev bought for {:.2} SOL\t\t*Mint: {}",
            buy_event.sol_amount as f64 / 10f64.powi(9),
            token_data.token_mint
        );
        token_data.dev_buy_amount_sol = buy_event.sol_amount as f64 / 10f64.powi(9);
    }

    if let Some(existing_holder_amount) = token_data.token_holders.get(&buy_event.user) {
        let updated_holder_amount = existing_holder_amount + buy_event.token_amount;
        token_data
            .token_holders
            .insert(buy_event.user, updated_holder_amount);
    } else {
        token_data
            .token_holders
            .insert(buy_event.user, buy_event.token_amount);
    };

    //update sell state flag
    token_data.update_sell_state_flag(tx_id.clone());

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Buy".green(),
            tx_id,
            buy_event.mint.to_string()
        );
        token_data.token_is_purchased = true;
        token_data.token_average_buying_price = (buy_event.sol_amount as f64 / 10f64.powi(9))
            / (buy_event.token_amount as f64 / 10f64.powi(6));
        token_data.token_balance += buy_event.token_amount;

        token_data.tp_selling_plan = TPSellingPlan {
            tp_1: (*TAKE_PROFIT_1_PCNT * (token_data.token_balance as f64)) as u64,
            tp_2: (*TAKE_PROFIT_2_PCNT * (token_data.token_balance as f64)) as u64,
            tp_3: (*TAKE_PROFIT_3_PCNT * (token_data.token_balance as f64)) as u64,
            tp_4: (*TAKE_PROFIT_4_PCNT * (token_data.token_balance as f64)) as u64,
            tp_5: (*TAKE_PROFIT_5_PCNT * (token_data.token_balance as f64)) as u64,
        };

        token_data.ts_stop_selling_plan = TSStopSellingPlan {
            ts_1_stop: (*TS_1_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_2_stop: (*TS_2_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_3_stop: (*TS_3_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_4_stop: (*TS_4_SELL_PCNT * (token_data.token_balance as f64)) as u64,
            ts_5_stop: (*TS_5_SELL_PCNT * (token_data.token_balance as f64)) as u64,
        };

        token_data.sl_selling_plan = SLSellingPlan {
            sl_1: (*STOP_LOSS_1_PCNT * (token_data.token_balance as f64)) as u64,
            sl_2: (*STOP_LOSS_3_PCNT * (token_data.token_balance as f64)) as u64,
            sl_3: (*STOP_LOSS_2_PCNT * (token_data.token_balance as f64)) as u64,
        };

        update!(
            "Mint: {}\t*TSStopSellingPlan: {:#?}\t*TPSellingPlan {:#?}",
            buy_event.mint.to_string(),
            token_data.tp_selling_plan,
            token_data.ts_stop_selling_plan
        );
    }
    let _ = TOKEN_DB.upsert(buy_event.mint.clone(), token_data.clone());
    token_data.clone()
}

pub fn update_status_from_sell_event(
    mut token_data: TokenDatabaseSchema,
    sell_event: SellEvent,
    tx_id: String,
) -> Option<TokenDatabaseSchema> {
    let updated_token_price = (sell_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (sell_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    info!(
        "[Token Sell]\t*Mint: {}\t*User: {}\t*MC: {:.2}\t*Amount: {:.2}",
        token_data.token_mint,
        sell_event.user,
        token_data.token_marketcap,
        sell_event.sol_amount as f64 / 10f64.powi(9)
    );

    token_data.token_volume = if let Some(val) = token_data.token_volume {
        Some(val + sell_event.sol_amount as f64 / 10f64.powi(9))
    } else {
        None
    };

    if let Some(existing_holder_amount) = token_data.token_holders.get(&sell_event.user) {
        if *existing_holder_amount > sell_event.token_amount {
            let updated_holder_amount = existing_holder_amount - sell_event.token_amount;
            token_data
                .token_holders
                .insert(sell_event.user, updated_holder_amount);
        } else {
            token_data.token_holders.remove(&sell_event.user);
        };
    }

    //update sell state flag
    token_data.update_sell_state_flag(tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Sell".red(),
            tx_id,
            sell_event.mint.to_string()
        );
        token_data.token_balance -= sell_event.token_amount;

        if token_data.token_balance > 0 {
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let _ = TOKEN_DB.delete(sell_event.mint.clone());
            None
        }
    } else {
        let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
        Some(token_data.clone())
    }
}

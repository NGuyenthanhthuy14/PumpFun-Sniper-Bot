use std::time::{Duration};

use crate::*;
use colored::*;

pub fn update_status_from_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: BuyEvent,
    _is_bundler_buy: bool,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));

    token_data.token_max_price = token_data.token_max_price.max(updated_token_price);
    token_data.token_price = updated_token_price;
    token_data.token_marketcap = updated_token_price * token_data.token_total_supply as f64;

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[My Tx]\t[{}]\t*Hash: {}\t*mint: {}",
            "Buy".green(),
            tx_id,
            buy_event.mint
        );
        token_data.token_is_purchased = true;
        token_data.token_balance += buy_event.token_amount;
        token_data.token_average_buying_price = updated_token_price;

        token_data.tp_selling_plan = TPSellingPlan {
            tp_1: (*TAKE_PROFIT_1_PCNT * token_data.token_balance as f64) as u64,
            tp_2: (*TAKE_PROFIT_2_PCNT * token_data.token_balance as f64) as u64,
            tp_3: (*TAKE_PROFIT_3_PCNT * token_data.token_balance as f64) as u64,
            tp_4: (*TAKE_PROFIT_4_PCNT * token_data.token_balance as f64) as u64,
            tp_5: (*TAKE_PROFIT_5_PCNT * token_data.token_balance as f64) as u64,
        };
    } else if buy_event.user == token_data.token_creator {
        if token_data.dev_buy_sol_lamports == None {
            info!(
                "[Dev Buy]\t*amount: {:.9} SOL*mint: {}",
                buy_event.sol_amount as f64 / 10f64.powi(9),
                buy_event.mint
            );
            token_data.dev_buy_sol_lamports = Some(buy_event.sol_amount);
            token_data.token_price_after_mint_bundler = updated_token_price;
        };
    } else if token_data.token_mint_time.elapsed() < Duration::from_millis(40) {
        //Token mint bundler transactions
        info!(
            "[Mint Bundler]\t*amount: {:.3} SOL*mint: {}",
            buy_event.sol_amount as f64 / 10f64.powi(9),
            buy_event.mint
        );
        token_data.token_price_after_mint_bundler = updated_token_price;
    }

    //update sell state flag
    token_data.update_sell_state_flag(tx_id.clone());

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

    //update buy state flag
    token_data.update_buy_state_flag();
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

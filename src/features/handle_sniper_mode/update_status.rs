use crate::*;

pub fn update_status_from_buy_event(
    mut token_data: TokenDatabaseSchema,
    buy_event: BuyEvent,
    tx_id: String,
) -> TokenDatabaseSchema {
    let updated_token_price = (buy_event.virtual_sol_reserves as f64 / 10f64.powi(9))
        / (buy_event.virtual_token_reserves as f64 / 10f64.powi(6));
        
    token_data.token_event = TokenEventType::BuyTokenEvent;
    token_data.update_status(updated_token_price, tx_id.clone());

    if buy_event.user == *SIGNER_PUBKEY {
        info!(
            "[MY_TRADE] => HASH : {}
                \t* TRADE : BUY
                \t* MINT : {}",
            tx_id,
            buy_event.mint.to_string()
        );
        token_data.token_is_purchased = true;
        token_data.token_buying_point_price = (buy_event.sol_amount as f64 / 10f64.powi(9))
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

        update!(
            "MINT : {}
            \t* TSStopSellingPlan : {:#?}
            \t* TPSellingPlan {:#?}",
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

    token_data.token_event = TokenEventType::SellTokenEvent;
    token_data.update_status(updated_token_price, tx_id.clone());

    if sell_event.user == *SIGNER_PUBKEY {
        info!(
            "[MY_TRADE] => HASH : {}
                \t* TRADE : SELL
                \t* MINT : {}",
            tx_id,
            sell_event.mint.to_string()
        );
        token_data.token_balance -= sell_event.token_amount;

        if token_data.token_balance > 0 {
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

            update!(
                "MINT : {}
                \t* TSStopSellingPlan : {:#?}
                \t* TPSellingPlan {:#?}",
                sell_event.mint.to_string(),
                token_data.tp_selling_plan,
                token_data.ts_stop_selling_plan
            );
            let _ = TOKEN_DB.upsert(sell_event.mint.clone(), token_data.clone());
            Some(token_data.clone())
        } else {
            let _ = TOKEN_DB.delete(sell_event.mint.clone());
            None
        }
    } else {
        Some(token_data.clone())
    }
}

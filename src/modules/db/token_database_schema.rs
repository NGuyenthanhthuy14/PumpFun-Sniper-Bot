use crate::*;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone, Debug)]
pub struct TokenDatabaseSchema {
    pub token_mint: Pubkey,
    pub token_price: f64,
    pub token_peak_price: f64,
    pub token_balance: u64,
    pub token_buying_point_price: f64,
    pub token_event: TokenEventType,
    pub token_is_purchased: bool,
    pub tp_states: TPMode,
    pub ts_states: TSMode,
    pub ts_stop_selling_plan: TSStopSellingPlan,
    pub tp_selling_plan: TPSellingPlan,
    pub pump_fun_swap_accounts: PumpFunSwapAccounts,
}

impl TokenDatabaseSchema {
    pub fn new_from_mint(
        mint_event: MintEvent,
        mint_instruction_accounts: MintInstructionAccounts,
        tx_id: String,
    ) -> Self {
        info!(
            "[NEW_MINT] => MINT : {}
            \t* TX HASH : {:?}",
            mint_event.mint.to_string(),
            solscan!(tx_id)
        );
        let token_data = Self {
            token_mint: mint_event.mint,
            token_balance: 0,
            token_price: (mint_event.virtual_sol_reserves as f64 / 10f64.powi(9))
                / (mint_event.virtual_token_reserves as f64 / 10f64.powi(6)),
            token_peak_price: (mint_event.virtual_sol_reserves as f64 / 10f64.powi(9))
                / (mint_event.virtual_token_reserves as f64 / 10f64.powi(6)),
            token_buying_point_price: 0.0,
            token_event: TokenEventType::MintTokenEvent,
            token_is_purchased: false,
            tp_states: TPMode::None,
            ts_states: TSMode::None,
            tp_selling_plan: TPSellingPlan {
                tp_1: 0,
                tp_2: 0,
                tp_3: 0,
                tp_4: 0,
                tp_5: 0,
            },
            ts_stop_selling_plan: TSStopSellingPlan {
                ts_1_stop: 0,
                ts_2_stop: 0,
                ts_3_stop: 0,
                ts_4_stop: 0,
                ts_5_stop: 0,
            },
            pump_fun_swap_accounts: PumpFunSwapAccounts::from_mint(
                &mint_instruction_accounts,
                &mint_event,
            ),
        };
        let _ = TOKEN_DB.upsert(mint_event.mint.clone(), token_data.clone());
        token_data
    }

    pub fn update_status(&mut self, updated_token_price: f64, tx_id: String) {
        self.token_price = updated_token_price;
        self.token_peak_price = self.token_price.max(updated_token_price);

        if self.token_balance > 0 {
            self.tp_states = if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_5
                && self.tp_states < TPMode::TP5
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::TP5,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::TP5
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_4
                && self.tp_states < TPMode::TP4
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::TP4,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::TP4
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_3
                && self.tp_states < TPMode::TP3
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::TP4,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::TP3
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_2
                && self.tp_states < TPMode::TP2
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::TP2,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::TP2
            } else if self.token_price > self.token_buying_point_price * *TAKE_PROFIT_1
                && self.tp_states < TPMode::TP1
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::TP1,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::TP1
            } else if self.token_price < self.token_buying_point_price * *STOP_LOSS
                && self.tp_states < TPMode::SL
            {
                info!(
                    "[TP_UPDATED] => MINT : {}
                    \t* TP STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_states,
                    TPMode::SL,
                    self.token_buying_point_price,
                    self.token_price,
                );
                TPMode::SL
            } else {
                self.tp_states.clone()
            };

            self.ts_states = if self.ts_states == TSMode::TS5Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_5_STOP)
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS5Stop,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS5Stop
            } else if self.ts_states == TSMode::TS4Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_4_STOP)
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS4Stop,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS4Stop
            } else if self.ts_states == TSMode::TS3Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_3_STOP)
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS3Stop,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS3Stop
            } else if self.ts_states == TSMode::TS2Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_2_STOP)
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS2Stop,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS2Stop
            } else if self.ts_states == TSMode::TS1Triggered
                && self.token_price < self.token_peak_price * (1.0 - *TS_1_STOP)
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS1Stop,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS1Stop
            } else if self.token_price > self.token_buying_point_price * *TS_5
                && self.ts_states < TSMode::TS5Triggered
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS5Triggered,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS5Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_4
                && self.ts_states < TSMode::TS4Triggered
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS4Triggered,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS4Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_3
                && self.ts_states < TSMode::TS3Triggered
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS3Triggered,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS3Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_2
                && self.ts_states < TSMode::TS2Triggered
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS2Triggered,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS2Triggered
            } else if self.token_price > self.token_buying_point_price * *TS_1
                && self.ts_states < TSMode::TS1Triggered
            {
                info!(
                    "[TS_UPDATED] => MINT : {}
                    \t* TS STATE : {:?} -> {:?},
                    \t* PRICE VARIANT : {} (BUY) -> {} (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.ts_states,
                    TSMode::TS1Triggered,
                    self.token_buying_point_price,
                    self.token_price
                );
                TSMode::TS1Triggered
            } else {
                self.ts_states.clone()
            };

            dev_log!(
                "[POOL STATE UPDATE] => MINT {:<12} ,
                \t* TX HASH : {}
                \t* CURRENT PRICE : {:<12} , PEAK PRICE : {:<12} , BUYING POINT PRICE : {:<12}
                \t* PRICE VARIANT PCNT : {:3.5} % , FALL PCNT : {:3.5} %
                \t* ts_states : {:?} , tp_states : {:?}",
                &self.pump_fun_swap_accounts.mint.to_string(),
                solscan!(tx_id),
                &self.token_price,
                &self.token_peak_price,
                &self.token_buying_point_price,
                &self.token_price * 100.0 / &self.token_buying_point_price,
                100.0 * (&self.token_peak_price - &self.token_price) / &self.token_peak_price,
                self.ts_states,
                self.tp_states,
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TSMode {
    None,
    TS1Triggered,
    TS1Stop,
    TS2Triggered,
    TS2Stop,
    TS3Triggered,
    TS3Stop,
    TS4Triggered,
    TS4Stop,
    TS5Triggered,
    TS5Stop,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TPMode {
    None,
    TP1,
    TP2,
    TP3,
    TP4,
    TP5,
    SL,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenEventType {
    MintTokenEvent,
    BuyTokenEvent,
    SellTokenEvent
}

#[derive(Debug, Clone, Copy)]
pub struct TSStopSellingPlan {
    pub ts_1_stop: u64,
    pub ts_2_stop: u64,
    pub ts_3_stop: u64,
    pub ts_4_stop: u64,
    pub ts_5_stop: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TPSellingPlan {
    pub tp_1: u64,
    pub tp_2: u64,
    pub tp_3: u64,
    pub tp_4: u64,
    pub tp_5: u64,
}

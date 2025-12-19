use crate::*;
use colored::*;
use solana_sdk::pubkey::Pubkey;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct TokenDatabaseSchema {
    pub token_mint: Pubkey,
    pub token_mint_time: Instant,
    pub token_creator: Pubkey,
    pub token_total_supply: u64,
    pub token_price: f64,
    pub token_max_price: f64,
    pub token_is_purchased: bool,
    pub token_balance: u64,
    pub token_average_buying_price: f64,
    pub token_marketcap: f64,
    pub tp_state: TPMode,
    pub tracked_tp_state: TPMode,
    pub tp_selling_plan: TPSellingPlan,
    pub pump_fun_swap_accounts: PumpFunSwapAccounts,
    pub token_sniper_status: TokenSniperStatus,
    pub token_trade_signal: TokenTradeSignal,
    pub token_sell_status: TokenSellStatus,
    pub mint_budget_compute_unit_limit: u32,
    pub mint_budget_compute_unit_price: u64,
    pub dev_buy_sol_lamports: Option<u64>,
    pub token_price_after_mint_bundler: f64,
}

impl TokenDatabaseSchema {
    pub fn new_from_mint(
        mint_event: MintEvent,
        mint_instruction_accounts: MintInstructionAccounts,
        budget_compute_data: (u32, u64),
        _tx_id: String,
    ) -> Self {
        info!(
            "[{}]\t\t\t*Mint: {}\t*Creator: {}\t*BudgetComputeUnitLimit: {}\t*BudgetComputeUnitPrice: {}",
            "Mint".blue(),
            mint_event.mint.to_string(),
            mint_event.creator,
            budget_compute_data.0,
            budget_compute_data.1
        );

        let initial_token_price = (mint_event.virtual_sol_reserves as f64 / 10f64.powi(9))
            / (mint_event.virtual_token_reserves as f64 / 10f64.powi(6));
        let initial_token_marketcap = initial_token_price * mint_event.token_total_supply as f64;

        let token_data = Self {
            token_mint: mint_event.mint,
            token_mint_time: Instant::now(),
            token_creator: mint_event.creator,
            token_total_supply: mint_event.token_total_supply / 10u64.pow(6),
            token_balance: 0,
            token_price: initial_token_price,
            token_max_price: initial_token_price,
            token_is_purchased: false,
            token_marketcap: initial_token_marketcap,
            token_average_buying_price: 0.0,
            tp_state: TPMode::None,
            tracked_tp_state: TPMode::None,
            tp_selling_plan: TPSellingPlan {
                tp_1: 0,
                tp_2: 0,
                tp_3: 0,
                tp_4: 0,
                tp_5: 0,
            },
            pump_fun_swap_accounts: PumpFunSwapAccounts::from_mint(
                &mint_instruction_accounts,
                &mint_event,
            ),
            token_sniper_status: TokenSniperStatus::TokenMinted,
            token_trade_signal: TokenTradeSignal::None,
            mint_budget_compute_unit_limit: budget_compute_data.0,
            mint_budget_compute_unit_price: budget_compute_data.1,
            dev_buy_sol_lamports: None,
            token_price_after_mint_bundler: 0.0,
            token_sell_status: TokenSellStatus::None,
        };

        let _ = TOKEN_DB.upsert(mint_event.mint.clone(), token_data.clone());
        token_data
    }

    pub fn update_buy_state_flag(&mut self) {
        //Buy token when token price goes lower than 0.8 * mint bundle price
        let dev_buy_amount_filtered = dev_buy_filter(self.dev_buy_sol_lamports);
        if dev_buy_amount_filtered
            && !self.token_is_purchased
            && self.token_sell_status != TokenSellStatus::SellTradeSubmitted
            && self.token_price_after_mint_bundler != 0.0
            && self.token_price <= self.token_price_after_mint_bundler * 0.8
            && self.token_mint_time.elapsed() < Duration::from_secs(300)
        {
            info!("Token price is lower than initial bundle price * 0.8, buying...");
            self.token_trade_signal = TokenTradeSignal::IsEntryPoint;
        }
    }

    pub fn update_sell_state_flag(&mut self, tx_id: String) {
        //Works for TP4, TP5
        if self.token_balance > 0 {
            self.tp_state = if self.token_price > self.token_average_buying_price * *TAKE_PROFIT_5
                && self.tp_state < TPMode::TP5
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP5,
                    self.token_average_buying_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP5
            } else if self.token_price > self.token_average_buying_price * *TAKE_PROFIT_4
                && self.tp_state < TPMode::TP4
            {
                update!(
                    "[TP_UPDATED]\t*MINT: {}
                    \t*TP STATE: {:?} -> {:?},
                    \t*MC VARIANT: {} SOL (BUY) -> {} SOL (NOW)",
                    self.pump_fun_swap_accounts.mint,
                    self.tp_state,
                    TPMode::TP4,
                    self.token_average_buying_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                    self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                );
                TPMode::TP4
            } else {
                self.tp_state.clone()
            };

            dev_log!(
                "[POOL STATE UPDATE]\t*MINT {:<12} ,
                \t*TX HASH: {}
                \t*CURRENT MC: {:.5} SOL , PEAK MC: {:.5} SOL, BUYING POINT MC: {:.5} SOL
                \t*PRICE VARIANT PCNT: {:3.5} % , FALL PCNT: {:3.5} %
                \t*tp_state: {:?}
                \t*amount: {}",
                &self.pump_fun_swap_accounts.mint.to_string(),
                solscan!(tx_id),
                &self.token_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_max_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_average_buying_price * PUMP_FUN_TOKEN_TOTAL_SUPPLY as f64,
                &self.token_price * 100.0 / &self.token_average_buying_price,
                100.0 * (&self.token_max_price - &self.token_price) / &self.token_max_price,
                self.tp_state,
                self.token_balance
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
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum SLMode {
    None,
    SL1,
    SL2,
    SL3,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenSniperStatus {
    None,
    TokenMinted,
    SniperTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenCopyTradeStatus {
    None,
    TargetBought,
    TargetSold,
    CopyTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenSellStatus {
    None,
    SellTradeSubmitted,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenEvent {
    MintTokenEvent,
    BuyTokenEvent,
    SellTokenEvent,
}

#[derive(Debug, Clone, Copy)]
pub struct SLSellingPlan {
    pub sl_1: u64,
    pub sl_2: u64,
    pub sl_3: u64,
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

#[derive(Debug, Clone)]
pub struct LastEvent {
    pub tx_hash: String,
    pub last_tracked_event: TokenEvent,
    pub last_activity_timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct TokenHoldersInfo {
    pub holder_accounts: Vec<Pubkey>,
    pub max_holder: Option<Pubkey>,
    pub max_holder_percent: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenPriceTrending {
    None,
    Rising,
    Falling,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Copy)]
pub enum TokenTradeSignal {
    None,
    IsEntryPoint,
    EntrySubmitted,
    IsExitPoint,
    ExitSubmitted,
}

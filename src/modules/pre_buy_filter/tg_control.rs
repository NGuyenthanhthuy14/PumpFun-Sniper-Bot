use crate::*;
use serde_json::{json, Value};
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

pub static WARN_ONLY_MODE: AtomicBool = AtomicBool::new(false);
pub static ENABLE_M1_HOLDER: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M2_PANIC: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M3_DEV: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M4_GENESIS: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M5_METADATA: AtomicBool = AtomicBool::new(true);
pub static LIVE_HOLDER_PCT: AtomicU64 = AtomicU64::new(30);
pub static LIVE_DEV_AGE: AtomicU64 = AtomicU64::new(6);
pub static LIVE_DEV_TXS: AtomicU64 = AtomicU64::new(10);
pub static MASTER_SWITCH: AtomicBool = AtomicBool::new(true);

// Global State
pub static BOT_IS_RUNNING: AtomicBool = AtomicBool::new(false);

// Live buy amount (in lamports, divide by 1e9 to get SOL)
// 0 means use default from Config.toml
pub static LIVE_BUY_AMOUNT_LAMPORTS: AtomicU64 = AtomicU64::new(0);

/// Get current buy amount in SOL (live override or config default)
pub fn get_live_buy_amount_sol() -> f64 {
    let live = LIVE_BUY_AMOUNT_LAMPORTS.load(Ordering::Relaxed);
    if live > 0 {
        live as f64 / 1_000_000_000.0
    } else {
        *crate::BUY_AMOUNT_SOL
    }
}

// Live Stop Loss (stored as percent * 100, e.g. 5000 = 50%)
// 0 means use default from Config.toml
pub static LIVE_STOP_LOSS_BPS: AtomicU64 = AtomicU64::new(0);

/// Get current stop loss multiplier (e.g. 0.50 = 50% of buy price)
pub fn get_live_stop_loss() -> f64 {
    let bps = LIVE_STOP_LOSS_BPS.load(Ordering::Relaxed);
    if bps > 0 {
        bps as f64 / 10_000.0
    } else {
        *crate::STOP_LOSS
    }
}

// Live Max Risk Score (0 = use config default)
pub static LIVE_MAX_RISK: AtomicU64 = AtomicU64::new(0);

/// Get current max risk score
pub fn get_live_max_risk() -> f64 {
    let r = LIVE_MAX_RISK.load(Ordering::Relaxed);
    if r > 0 { r as f64 } else { *crate::MAX_TOTAL_RISK_SCORE }
}

// Live Dynamic Sizing toggle (0=unset, 1=on, 2=off)
pub static LIVE_DYNAMIC_SIZING: AtomicU64 = AtomicU64::new(0);

/// Get current dynamic sizing state
pub fn get_live_dynamic_sizing() -> bool {
    match LIVE_DYNAMIC_SIZING.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => *crate::ENABLE_DYNAMIC_SIZING,
    }
}

// Live TP Trailing (stored as multiplier * 1000, e.g. 2000 = 2.0x)
pub static LIVE_TP_TRAILING: AtomicU64 = AtomicU64::new(0);

/// Get current TP trailing multiplier (e.g. 2.0 = sell at 2x buy price)
pub fn get_live_tp_trailing() -> f64 {
    let v = LIVE_TP_TRAILING.load(Ordering::Relaxed);
    if v > 0 { v as f64 / 1_000.0 } else { *crate::TP_TRAILING }
}

// Live Trailing Stop (stored as multiplier * 1000, e.g. 850 = 0.85)
pub static LIVE_TRAILING_STOP: AtomicU64 = AtomicU64::new(0);

/// Get current trailing stop multiplier (e.g. 0.85 = sell if price drops 15% from peak)
pub fn get_live_trailing_stop() -> f64 {
    let v = LIVE_TRAILING_STOP.load(Ordering::Relaxed);
    if v > 0 { v as f64 / 1_000.0 } else { *crate::TRAILING_STOP }
}

// Input state: track what the user is currently setting via free text
// 0=none, 1=buy_amount, 2=stop_loss, 3=max_risk, 4=import_key, 5=tp_trailing, 6=trailing_stop
pub static INPUT_STATE: AtomicU64 = AtomicU64::new(0);

// Global Stats (Mocked for now, wired up later)
pub static STAT_SCANNED: AtomicU64 = AtomicU64::new(0);
pub static STAT_PASSED: AtomicU64 = AtomicU64::new(0);
pub static STAT_REJECTED: AtomicU64 = AtomicU64::new(0);
pub static STAT_WARNED: AtomicU64 = AtomicU64::new(0);
pub static STAT_SKIPPED: AtomicU64 = AtomicU64::new(0);
pub static STAT_REALIZED_PNL: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
pub static STAT_TOTAL_SPENT: AtomicU64 = AtomicU64::new(0);
pub static STAT_TOTAL_RECEIVED: AtomicU64 = AtomicU64::new(0);
pub static STAT_WINS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static STAT_LOSSES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static STAT_BUYS_ATTEMPTED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static STAT_BUYS_SUCCESS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static STAT_BUYS_FAILED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub static STAT_TOTAL_SELLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub async fn start_telegram_control_bot() {
    let token = match env::var("TG_BOT_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return,
    };
    let chat_id = match env::var("TG_CHAT_ID") {
        Ok(c) if !c.is_empty() => c,
        _ => return,
    };

    let client = reqwest::Client::new();
    let mut offset: i64 = 0;

    info!("🚀 Telegram Control Bot Task Started...");

    // ── Flush all pending updates so we don't replay old messages after restart ──
    let flush_url = format!("https://api.telegram.org/bot{}/getUpdates?offset=-1&limit=1", token);
    if let Ok(resp) = client.get(&flush_url).send().await {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(results) = json["result"].as_array() {
                if let Some(last) = results.last() {
                    if let Some(update_id) = last["update_id"].as_i64() {
                        offset = update_id + 1;
                        info!("🧹 [TG_CONTROL] Flushed old updates, starting from offset {}", offset);
                    }
                }
            }
        }
    }

    loop {
        let url = format!("https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=30", token, offset);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(results) = json["result"].as_array() {
                    for result in results {
                        if let Some(update_id) = result["update_id"].as_i64() {
                            offset = update_id + 1;
                        }

                        // Handle message
                        if let Some(message) = result.get("message") {
                            let text = message["text"].as_str().unwrap_or("");
                            let sender_chat_id = message["chat"]["id"].as_i64().unwrap_or(0).to_string();
                            
                            info!("📥 [TG_CONTROL] Received msg: '{}' from chat_id: {}", text, sender_chat_id);
                            
                            if sender_chat_id == chat_id {
                                handle_text_message(&client, &token, &chat_id, text).await;
                            } else {
                                info!("⚠️ [TG_CONTROL] Ignored msg from unauthorized chat_id: {} (expected: {})", sender_chat_id, chat_id);
                            }
                        }

                        // Handle callback query
                        if let Some(callback) = result.get("callback_query") {
                            let data = callback["data"].as_str().unwrap_or("");
                            let callback_id = callback["id"].as_str().unwrap_or("");
                            let sender_chat_id = callback["message"]["chat"]["id"].as_i64().unwrap_or(0).to_string();
                            let message_id = callback["message"]["message_id"].as_i64().unwrap_or(0);

                            if sender_chat_id == chat_id {
                                handle_callback(&client, &token, &chat_id, message_id, data).await;
                                let answer_url = format!("https://api.telegram.org/bot{}/answerCallbackQuery", token);
                                let _ = client.post(&answer_url).json(&json!({"callback_query_id": callback_id})).send().await;
                            }
                        }
                    }
                } else if let Some(desc) = json.get("description") {
                    error!("❌ [TG_CONTROL] API Error: {}", desc);
                }
            } else {
                error!("❌ [TG_CONTROL] Failed to parse JSON response");
            }
        } else {
            error!("❌ [TG_CONTROL] HTTP request failed");
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_text_message(client: &reqwest::Client, token: &str, chat_id: &str, text: &str) {
    if text.starts_with("/start") || text.contains("Dashboard") {
        send_dashboard(client, token, chat_id, "Today").await;
    } else if text.contains("Wallet management") || text.starts_with("/wallets") {
        send_wallet_menu(client, token, chat_id).await;
    } else if text.starts_with("/generate") {
        handle_generate_wallet(client, token, chat_id).await;
    } else if text.starts_with("/import_key") {
        INPUT_STATE.store(4, Ordering::Relaxed);
        send_simple_msg_with_parse_mode(client, token, chat_id,
            "🔑 <b>Import Wallet</b>\n\nPaste your <b>base58 private key</b> in the next message:\n\n<i>⚠️ The key will NOT be sent to any external server. It is stored locally on the VPS only.</i>",
            "HTML"
        ).await;
    } else if text.starts_with("/select_") {
        if let Ok(idx) = text[8..].trim().parse::<usize>() {
            handle_select_wallet(client, token, chat_id, idx).await;
        } else {
            send_simple_msg(client, token, chat_id, "❌ Invalid. Use /select_1, /select_2, etc.").await;
        }
    } else if text.starts_with("/delete_") {
        if let Ok(idx) = text[8..].trim().parse::<usize>() {
            handle_delete_wallet(client, token, chat_id, idx).await;
        } else {
            send_simple_msg(client, token, chat_id, "❌ Invalid. Use /delete_1, /delete_2, etc.").await;
        }
    } else if text.starts_with("/show_key_") {
        if let Ok(idx) = text[10..].trim().parse::<usize>() {
            handle_show_key(client, token, chat_id, idx).await;
        } else {
            send_simple_msg(client, token, chat_id, "❌ Invalid. Use /show_key_1, /show_key_2, etc.").await;
        }
    } else if text.contains("Trading parameters") {
        let current_buy = get_live_buy_amount_sol();
        let current_sl = get_live_stop_loss() * 100.0;
        let current_risk = get_live_max_risk();
        let current_dynamic = get_live_dynamic_sizing();
        let current_tp = get_live_tp_trailing();
        let current_trail = get_live_trailing_stop() * 100.0;
        let msg = format!("⚙️ Trading Parameters\n───────────────\n💸 Buy Amount: {:.4} SOL\n🛑 Stop Loss: {:.0}%\n🎯 Take Profit: {:.1}x\n📉 Trailing Stop: {:.0}%\n📈 Dynamic Sizing: {}\n🛡️ Max Risk Score: {:.0}\n\nTap any button to change live!",
            current_buy, current_sl, current_tp, current_trail,
            if current_dynamic { "✅ ON" } else { "❌ OFF" },
            current_risk
        );
        let keyboard = json!({
            "inline_keyboard": [
                [{"text": format!("💸 Buy Amount: {:.4} SOL", current_buy), "callback_data": "menu_buy_amount"}],
                [{"text": format!("🛑 Stop Loss: {:.0}%", current_sl), "callback_data": "menu_stop_loss"}],
                [{"text": format!("🎯 Take Profit: {:.1}x", current_tp), "callback_data": "menu_tp"}],
                [{"text": format!("📉 Trailing Stop: {:.0}%", current_trail), "callback_data": "menu_trailing"}],
                [{"text": format!("🛡️ Max Risk: {:.0}", current_risk), "callback_data": "menu_max_risk"}],
                [{"text": format!("📈 Dynamic Sizing: {}", if current_dynamic { "✅ ON" } else { "❌ OFF" }), "callback_data": "toggle_dynamic"}],
                [{"text": "🔙 Close", "callback_data": "ignore"}]
            ]
        });
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let payload = json!({ "chat_id": chat_id, "text": msg, "reply_markup": keyboard });
        let _ = client.post(&url).json(&payload).send().await;
    } else if text.contains("Anti-Rug") {
        send_settings_menu(client, token, chat_id).await;
    } else if text.starts_with("/stats") || text.contains("Dashboard") {
        send_dashboard(client, token, chat_id, "Today").await;
    } else if text.contains("Start") {
        BOT_IS_RUNNING.store(true, Ordering::Relaxed);
        send_simple_msg_with_keyboard(client, token, chat_id, "✅ Bot is STARTED. Ready to snipe!").await;
    } else if text.contains("Stop") || text.starts_with("/stop") {
        BOT_IS_RUNNING.store(false, Ordering::Relaxed);
        
        // Count how many tokens are being monitored
        let held: usize = TOKEN_DB.map.iter()
            .filter(|e| e.value().token_is_purchased)
            .count();
        
        let msg = if held > 0 {
            format!("🛑 Bot STOPPED — no new buys.\n\n📊 Monitoring {} token(s) (SL/TP/timeout still active).\n\n💡 Press 📤 Sell All to dump everything.", held)
        } else {
            "🛑 Bot STOPPED. No tokens held.".to_string()
        };
        send_simple_msg_with_keyboard(client, token, chat_id, &msg).await;
    } else if text.starts_with("/sell_all") || text.contains("Sell All") {
        // Force-sell ALL held tokens immediately
        let keys: Vec<solana_sdk::pubkey::Pubkey> = TOKEN_DB
            .map
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
        let mut sell_count = 0u32;
        for token_key in keys {
            if let Ok(Some(mut token_data)) = TOKEN_DB.get(token_key) {
                if token_data.token_is_purchased 
                    && token_data.token_sell_status != TokenSellStatus::SellTradeSubmitted
                    && token_data.token_balance > 0
                {
                    token_data.token_sell_status = TokenSellStatus::SellTradeSubmitted;
                    let _ = TOKEN_DB.upsert(token_key, token_data.clone());
                    
                    let mut ix: Vec<solana_sdk::instruction::Instruction> = Vec::new();
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
                    
                    if !ix.is_empty() {
                        let mint = token_key;
                        let balance = token_data.token_balance;
                        let tag = format!(
                            "[SELL] SELL_ALL | MINT: {} | AMT: {}",
                            mint, balance
                        );
                        tokio::spawn(async move {
                            let _ = confirm_sell_with_retry(mint, balance, ix, tag).await;
                        });
                        sell_count += 1;
                    }
                }
            }
        }
        
        let msg = if sell_count > 0 {
            format!("📤 Selling all {} token(s)...", sell_count)
        } else {
            "✅ No tokens to sell.".to_string()
        };
        send_simple_msg_with_keyboard(client, token, chat_id, &msg).await;
    } else {
        let state = INPUT_STATE.load(Ordering::Relaxed);
        // State 4: waiting for private key import
        if state == 4 {
            INPUT_STATE.store(0, Ordering::Relaxed);
            handle_import_wallet(client, token, chat_id, text.trim()).await;
            return;
        }
        // Try parsing as number for buy/sl/risk inputs
        if let Ok(val) = text.trim().parse::<f64>() {
            match state {
                // Stop Loss input (enter percent, e.g. 50 = 50%)
                2 => {
                    if val > 0.0 && val <= 100.0 {
                        let bps = (val / 100.0 * 10_000.0) as u64;
                        LIVE_STOP_LOSS_BPS.store(bps, Ordering::Relaxed);
                        INPUT_STATE.store(0, Ordering::Relaxed);
                        let msg = format!("✅ Stop Loss set to <b>{:.0}%</b>\n\n💡 Live — no restart needed.", val);
                        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
                    } else {
                        send_simple_msg(client, token, chat_id, "❌ Invalid. Enter 1 - 100 (%)").await;
                    }
                }
                // Max Risk input (enter score, e.g. 60)
                3 => {
                    if val >= 0.0 && val <= 100.0 {
                        LIVE_MAX_RISK.store(val as u64, Ordering::Relaxed);
                        INPUT_STATE.store(0, Ordering::Relaxed);
                        let msg = format!("✅ Max Risk Score set to <b>{:.0}</b>\n\n💡 Live — no restart needed.", val);
                        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
                    } else {
                        send_simple_msg(client, token, chat_id, "❌ Invalid. Enter 0 - 100").await;
                    }
                }
                // TP Trailing input (enter multiplier, e.g. 2.0 = 2x)
                5 => {
                    if val > 1.0 && val <= 100.0 {
                        LIVE_TP_TRAILING.store((val * 1_000.0) as u64, Ordering::Relaxed);
                        INPUT_STATE.store(0, Ordering::Relaxed);
                        let msg = format!("✅ Take Profit set to <b>{:.1}x</b>\n\n💡 Live — no restart needed.", val);
                        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
                    } else {
                        send_simple_msg(client, token, chat_id, "❌ Invalid. Enter 1.1 - 100.0 (multiplier)").await;
                    }
                }
                // Trailing Stop input (enter percent, e.g. 85 = keep 85% of peak)
                6 => {
                    if val > 0.0 && val <= 100.0 {
                        LIVE_TRAILING_STOP.store((val / 100.0 * 1_000.0) as u64, Ordering::Relaxed);
                        INPUT_STATE.store(0, Ordering::Relaxed);
                        let msg = format!("✅ Trailing Stop set to <b>{:.0}%</b>\n\n💡 Live — no restart needed.", val);
                        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
                    } else {
                        send_simple_msg(client, token, chat_id, "❌ Invalid. Enter 1 - 100 (%)").await;
                    }
                }
                // Default: Buy Amount input
                _ => {
                    if val > 0.0 && val <= 10.0 {
                        let lamports = (val * 1_000_000_000.0) as u64;
                        LIVE_BUY_AMOUNT_LAMPORTS.store(lamports, Ordering::Relaxed);
                        let msg = format!("✅ Buy amount set to <b>{:.4} SOL</b>\n\n💡 Live — no restart needed.", val);
                        send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
                    } else {
                        send_simple_msg(client, token, chat_id, "❌ Invalid. Enter 0.0001 - 10.0 SOL").await;
                    }
                }
            }
        }
    }
}

async fn send_simple_msg(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({ "chat_id": chat_id, "text": msg });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn send_simple_msg_with_parse_mode(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str, parse_mode: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": parse_mode });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn send_simple_msg_with_keyboard(client: &reqwest::Client, token: &str, chat_id: &str, msg: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({
        "chat_id": chat_id,
        "text": msg,
        "reply_markup": build_reply_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

fn build_reply_keyboard() -> Value {
    let run_btn = if BOT_IS_RUNNING.load(Ordering::Relaxed) { "⏹️ Stop" } else { "▶️ Start" };
    json!({
        "keyboard": [
            [{"text": "📊 Dashboard"}],
            [{"text": "💰 Wallet management"}, {"text": "⚙️ Trading parameters"}],
            [{"text": "🛡️ Anti-Rug"}, {"text": run_btn}, {"text": "📤 Sell All"}]
        ],
        "resize_keyboard": true,
        "is_persistent": true
    })
}

fn build_dashboard_text(period: &str) -> String {
    let scanned = STAT_SCANNED.load(Ordering::Relaxed);
    let passed = STAT_PASSED.load(Ordering::Relaxed);
    let rejected = STAT_REJECTED.load(Ordering::Relaxed);
    let warned = STAT_WARNED.load(Ordering::Relaxed);
    let skipped = STAT_SKIPPED.load(Ordering::Relaxed);
    let pass_rate = if scanned > 0 { (passed as f64 / scanned as f64) * 100.0 } else { 0.0 };

    let pnl = STAT_REALIZED_PNL.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
    let pnl_icon = if pnl >= 0.0 { "🟢" } else { "🔴" };
    let pnl_sign = if pnl >= 0.0 { "+" } else { "" };
    
    let spent = STAT_TOTAL_SPENT.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
    let recv = STAT_TOTAL_RECEIVED.load(Ordering::Relaxed) as f64 / 1_000_000_000.0;
    
    let wins = STAT_WINS.load(Ordering::Relaxed);
    let losses = STAT_LOSSES.load(Ordering::Relaxed);
    let total_trades = wins + losses;
    let win_rate = if total_trades > 0 { (wins as f64 / total_trades as f64) * 100.0 } else { 0.0 };
    
    let buys_att = STAT_BUYS_ATTEMPTED.load(Ordering::Relaxed);
    let buys_succ = STAT_BUYS_SUCCESS.load(Ordering::Relaxed);
    let buys_fail = STAT_BUYS_FAILED.load(Ordering::Relaxed);
    let buy_rate = if buys_att > 0 { (buys_succ as f64 / buys_att as f64) * 100.0 } else { 0.0 };
    
    let sells = STAT_TOTAL_SELLS.load(Ordering::Relaxed);

    format!("📊 {}\n\
───────────────\n\
💰 PNL Summary\n\
├ {} Realized PNL: {}{:.4} SOL\n\
├ 💼 Total spent: {:.4} SOL\n\
├ 💸 Total received: {:.4} SOL\n\
├ 🏆 Win rate: {:.1}% ({}/{})\n\
├ ✅ Wins: {}\n\
└ ❌ Losses: {}\n\n\
💹 Trade Activity\n\
├ Total buys: {} (✅ {} / ❌ {})\n\
├ Buy success rate: {:.1}%\n\
└ Total sells: {}\n\n\
🛡️ Anti-Rug Filter\n\
├ Scanned: {}\n\
├ ✅ Passed: {}\n\
├ ❌ Rejected: {}\n\
├ ⚠️ Warned: {}\n\
├ 🚫 Skipped: {}\n\
└ Pass rate: {:.1}%", 
        period, 
        pnl_icon, pnl_sign, pnl, spent, recv, 
        win_rate, wins, total_trades, wins, losses,
        buys_att, buys_succ, buys_fail, buy_rate, sells,
        scanned, passed, rejected, warned, skipped, pass_rate
    )
}

fn build_dashboard_inline_keyboard() -> Value {
    json!({
        "inline_keyboard": [
            [{"text": "📊 Select time period for stats:", "callback_data": "ignore"}],
            [{"text": "📊 Today", "callback_data": "time_today"}, {"text": "📈 7 Days", "callback_data": "time_7d"}],
            [{"text": "📅 30 Days", "callback_data": "time_30d"}, {"text": "🌐 All Time", "callback_data": "time_all"}]
        ]
    })
}

async fn send_dashboard(client: &reqwest::Client, token: &str, chat_id: &str, period: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let text = build_dashboard_text(period);
    
    let payload = json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown",
        "reply_markup": build_dashboard_inline_keyboard()
    });
    
    // Send dashboard and simultaneously send the reply keyboard
    let _ = client.post(&url).json(&payload).send().await;
    
    // Ensure reply keyboard is attached
    let payload2 = json!({
        "chat_id": chat_id,
        "text": "Options loaded below 👇",
        "reply_markup": build_reply_keyboard()
    });
    let _ = client.post(&url).json(&payload2).send().await;
}

async fn update_dashboard(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64, period: &str) {
    let url = format!("https://api.telegram.org/bot{}/editMessageText", token);
    let text = build_dashboard_text(period);
    
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "text": text,
        "parse_mode": "Markdown",
        "reply_markup": build_dashboard_inline_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

// ── Anti-Rug Settings Logic ──

async fn handle_callback(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64, data: &str) {
    match data {
        "time_today" => update_dashboard(client, token, chat_id, message_id, "Today").await,
        "time_7d" => update_dashboard(client, token, chat_id, message_id, "7 Days").await,
        "time_30d" => update_dashboard(client, token, chat_id, message_id, "30 Days").await,
        "time_all" => update_dashboard(client, token, chat_id, message_id, "All Time").await,
        
        "menu_buy_amount" => {
            let current = get_live_buy_amount_sol();
            let keyboard = json!({
                "inline_keyboard": [
                    [{"text": "0.001 SOL", "callback_data": "buy_0.001"}, {"text": "0.005 SOL", "callback_data": "buy_0.005"}],
                    [{"text": "0.01 SOL", "callback_data": "buy_0.01"}, {"text": "0.05 SOL", "callback_data": "buy_0.05"}],
                    [{"text": "0.1 SOL", "callback_data": "buy_0.1"}, {"text": "0.5 SOL", "callback_data": "buy_0.5"}],
                    [{"text": "✏️ Custom (type amount)", "callback_data": "buy_custom"}],
                    [{"text": "🔙 Back", "callback_data": "ignore"}]
                ]
            });
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let msg = format!("💸 <b>Set Buy Amount</b>\n──────────────────\n📌 Current: <b>{:.4} SOL</b>\n\nSelect or type amount:", current);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML", "reply_markup": keyboard });
            let _ = client.post(&url).json(&payload).send().await;
        }
        
        "menu_stop_loss" => {
            let current_sl = get_live_stop_loss() * 100.0;
            let keyboard = json!({
                "inline_keyboard": [
                    [{"text": "20%", "callback_data": "sl_20"}, {"text": "30%", "callback_data": "sl_30"}, {"text": "40%", "callback_data": "sl_40"}],
                    [{"text": "50%", "callback_data": "sl_50"}, {"text": "60%", "callback_data": "sl_60"}, {"text": "70%", "callback_data": "sl_70"}],
                    [{"text": "✏️ Custom (type %)", "callback_data": "sl_custom"}],
                    [{"text": "🔙 Back", "callback_data": "ignore"}]
                ]
            });
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let msg = format!("🛑 <b>Set Stop Loss</b>\n──────────────────\n📌 Current: <b>{:.0}%</b>\n\nSelect or type %:", current_sl);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML", "reply_markup": keyboard });
            let _ = client.post(&url).json(&payload).send().await;
        }

        "menu_max_risk" => {
            let current_risk = get_live_max_risk();
            let keyboard = json!({
                "inline_keyboard": [
                    [{"text": "30", "callback_data": "risk_30"}, {"text": "50", "callback_data": "risk_50"}, {"text": "70", "callback_data": "risk_70"}],
                    [{"text": "80", "callback_data": "risk_80"}, {"text": "90", "callback_data": "risk_90"}, {"text": "100", "callback_data": "risk_100"}],
                    [{"text": "✏️ Custom (type value)", "callback_data": "risk_custom"}],
                    [{"text": "🔙 Back", "callback_data": "ignore"}]
                ]
            });
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let msg = format!("🛡️ <b>Set Max Risk Score</b>\n──────────────────\n📌 Current: <b>{:.0}</b>\n\nHigher = allow riskier tokens:", current_risk);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML", "reply_markup": keyboard });
            let _ = client.post(&url).json(&payload).send().await;
        }

        "menu_tp" => {
            let current_tp = get_live_tp_trailing();
            let keyboard = json!({
                "inline_keyboard": [
                    [{"text": "1.5x", "callback_data": "tp_1.5"}, {"text": "2.0x", "callback_data": "tp_2.0"}, {"text": "3.0x", "callback_data": "tp_3.0"}],
                    [{"text": "5.0x", "callback_data": "tp_5.0"}, {"text": "10.0x", "callback_data": "tp_10.0"}, {"text": "20.0x", "callback_data": "tp_20.0"}],
                    [{"text": "✏️ Custom (type multiplier)", "callback_data": "tp_custom"}],
                    [{"text": "🔙 Back", "callback_data": "ignore"}]
                ]
            });
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let msg = format!("🎯 <b>Set Take Profit</b>\n──────────────────\n📌 Current: <b>{:.1}x</b>\n\nSelect multiplier (e.g. 2.0 = sell at 2x buy price):", current_tp);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML", "reply_markup": keyboard });
            let _ = client.post(&url).json(&payload).send().await;
        }

        "menu_trailing" => {
            let current_trail = get_live_trailing_stop() * 100.0;
            let keyboard = json!({
                "inline_keyboard": [
                    [{"text": "70%", "callback_data": "trail_70"}, {"text": "80%", "callback_data": "trail_80"}, {"text": "85%", "callback_data": "trail_85"}],
                    [{"text": "90%", "callback_data": "trail_90"}, {"text": "95%", "callback_data": "trail_95"}],
                    [{"text": "✏️ Custom (type %)", "callback_data": "trail_custom"}],
                    [{"text": "🔙 Back", "callback_data": "ignore"}]
                ]
            });
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let msg = format!("📉 <b>Set Trailing Stop</b>\n──────────────────\n📌 Current: <b>{:.0}%</b>\n\nSell if price drops below this % of peak:", current_trail);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML", "reply_markup": keyboard });
            let _ = client.post(&url).json(&payload).send().await;
        }
        "toggle_dynamic" => {
            let current = get_live_dynamic_sizing();
            LIVE_DYNAMIC_SIZING.store(if current { 2 } else { 1 }, Ordering::Relaxed);
            let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
            let new_state = if current { "❌ OFF" } else { "✅ ON" };
            let msg = format!("📈 Dynamic Sizing: <b>{}</b>\n\n💡 Live — no restart needed.", new_state);
            let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
            let _ = client.post(&url).json(&payload).send().await;
        }

        "toggle_master" => {
            let current = MASTER_SWITCH.load(Ordering::Relaxed);
            MASTER_SWITCH.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "set_h30" => { LIVE_HOLDER_PCT.store(30, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_h40" => { LIVE_HOLDER_PCT.store(40, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_h50" => { LIVE_HOLDER_PCT.store(50, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_a0" => { LIVE_DEV_AGE.store(0, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_a3" => { LIVE_DEV_AGE.store(3, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_a6" => { LIVE_DEV_AGE.store(6, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_a12" => { LIVE_DEV_AGE.store(12, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_a24" => { LIVE_DEV_AGE.store(24, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_t0" => { LIVE_DEV_TXS.store(0, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_t5" => { LIVE_DEV_TXS.store(5, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "set_t10" => { LIVE_DEV_TXS.store(10, Ordering::Relaxed); update_settings_menu(client, token, chat_id, message_id).await; }
        "toggle_warn" => {
            let current = WARN_ONLY_MODE.load(Ordering::Relaxed);
            WARN_ONLY_MODE.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m1" => {
            let current = ENABLE_M1_HOLDER.load(Ordering::Relaxed);
            ENABLE_M1_HOLDER.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m2" => {
            let current = ENABLE_M2_PANIC.load(Ordering::Relaxed);
            ENABLE_M2_PANIC.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m3" => {
            let current = ENABLE_M3_DEV.load(Ordering::Relaxed);
            ENABLE_M3_DEV.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m4" => {
            let current = ENABLE_M4_GENESIS.load(Ordering::Relaxed);
            ENABLE_M4_GENESIS.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        "toggle_m5" => {
            let current = ENABLE_M5_METADATA.load(Ordering::Relaxed);
            ENABLE_M5_METADATA.store(!current, Ordering::Relaxed);
            update_settings_menu(client, token, chat_id, message_id).await;
        }
        _ => {
            if data.starts_with("buy_") {
                if data == "buy_custom" {
                    INPUT_STATE.store(0, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = "✏️ <b>Custom Buy Amount</b>\n\nType the amount in SOL (e.g. <code>0.005</code>):";
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                } else if let Ok(amount) = data[4..].parse::<f64>() {
                    LIVE_BUY_AMOUNT_LAMPORTS.store((amount * 1e9) as u64, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = format!("✅ Buy amount: <b>{:.4} SOL</b> 💡 Live", amount);
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                }
            } else if data.starts_with("sl_") {
                if data == "sl_custom" {
                    INPUT_STATE.store(2, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let payload = json!({ "chat_id": chat_id, "text": "✏️ <b>Custom Stop Loss</b>\n\nType percent (e.g. <code>45</code>):", "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                } else if let Ok(pct) = data[3..].parse::<f64>() {
                    LIVE_STOP_LOSS_BPS.store((pct / 100.0 * 10_000.0) as u64, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = format!("✅ Stop Loss: <b>{:.0}%</b> 💡 Live", pct);
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                }
            } else if data.starts_with("risk_") {
                if data == "risk_custom" {
                    INPUT_STATE.store(3, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let payload = json!({ "chat_id": chat_id, "text": "✏️ <b>Custom Max Risk</b>\n\nType score 0-100 (e.g. <code>60</code>):", "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                } else if let Ok(risk) = data[5..].parse::<f64>() {
                    LIVE_MAX_RISK.store(risk as u64, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = format!("✅ Max Risk Score: <b>{:.0}</b> 💡 Live", risk);
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                }
            } else if data.starts_with("tp_") {
                if data == "tp_custom" {
                    INPUT_STATE.store(5, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let payload = json!({ "chat_id": chat_id, "text": "✏️ <b>Custom Take Profit</b>\n\nType multiplier (e.g. <code>2.5</code> for 2.5x):", "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                } else if let Ok(mul) = data[3..].parse::<f64>() {
                    LIVE_TP_TRAILING.store((mul * 1_000.0) as u64, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = format!("✅ Take Profit: <b>{:.1}x</b> 💡 Live", mul);
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                }
            } else if data.starts_with("trail_") {
                if data == "trail_custom" {
                    INPUT_STATE.store(6, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let payload = json!({ "chat_id": chat_id, "text": "✏️ <b>Custom Trailing Stop</b>\n\nType percent (e.g. <code>85</code> = sell if drops below 85% of peak):", "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                } else if let Ok(pct) = data[6..].parse::<f64>() {
                    LIVE_TRAILING_STOP.store((pct / 100.0 * 1_000.0) as u64, Ordering::Relaxed);
                    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
                    let msg = format!("✅ Trailing Stop: <b>{:.0}%</b> 💡 Live", pct);
                    let payload = json!({ "chat_id": chat_id, "text": msg, "parse_mode": "HTML" });
                    let _ = client.post(&url).json(&payload).send().await;
                }
            }
        }
    }
}

fn build_settings_text() -> String {
    let master = if MASTER_SWITCH.load(Ordering::Relaxed) { "✅" } else { "❌" };
    let w = if WARN_ONLY_MODE.load(Ordering::Relaxed) { "⚠️ WARN ONLY — buys risky tokens" } else { "🛡️ BLOCK risky tokens" };
    let m1 = if ENABLE_M1_HOLDER.load(Ordering::Relaxed) { "✅" } else { "❌" };
    let m2 = if ENABLE_M2_PANIC.load(Ordering::Relaxed) { "✅" } else { "❌" };
    let m3 = if ENABLE_M3_DEV.load(Ordering::Relaxed) { "✅" } else { "❌" };
    let m4 = if ENABLE_M4_GENESIS.load(Ordering::Relaxed) { "✅" } else { "❌" };
    let m5 = if ENABLE_M5_METADATA.load(Ordering::Relaxed) { "✅" } else { "❌" };
    
    let h_pct = LIVE_HOLDER_PCT.load(Ordering::Relaxed);
    let d_age = LIVE_DEV_AGE.load(Ordering::Relaxed);
    let d_tx = LIVE_DEV_TXS.load(Ordering::Relaxed);

    format!("🛡️ Anti-Rug Intelligence\n\
───────────────\n\n\
{} Master Switch\n\
{}\n\n\
┌ Modules\n\
│ {} M1 — Holder Analyzer (top10 <= {}%)\n\
│ {} M2 — Panic-Sell Monitor\n\
│ {} M3 — Dev Wallet (age >= {}h, tx >= {})\n\
│ {} M4 — Genesis Detector\n\
└ {} M5 — Metadata Checker\n\n\
⏱️ Timeout: 1500ms • 💎 Jito tip: 0.0010 SOL", 
        master, w, m1, h_pct, m2, m3, d_age, d_tx, m4, m5
    )
}

fn build_settings_keyboard() -> Value {
    let h = LIVE_HOLDER_PCT.load(Ordering::Relaxed);
    let a = LIVE_DEV_AGE.load(Ordering::Relaxed);
    let t = LIVE_DEV_TXS.load(Ordering::Relaxed);

    let h30 = if h == 30 { "• 30%" } else { "30%" };
    let h40 = if h == 40 { "• 40%" } else { "40%" };
    let h50 = if h == 50 { "• 50%" } else { "50%" };

    let a0 = if a == 0 { "• 0h" } else { "0h" };
    let a3 = if a == 3 { "• 3h" } else { "3h" };
    let a6 = if a == 6 { "• 6h" } else { "6h" };
    let a12 = if a == 12 { "• 12h" } else { "12h" };
    let a24 = if a == 24 { "• 24h" } else { "24h" };

    let t0 = if t == 0 { "• 0 TXs" } else { "0 TXs" };
    let t5 = if t == 5 { "• 5 TXs" } else { "5 TXs" };
    let t10 = if t == 10 { "• 10 TXs" } else { "10 TXs" };

    json!({
        "inline_keyboard": [
            [{"text": "🔌 Master ON/OFF", "callback_data": "toggle_master"}, {"text": "⚠️ Warn / Block", "callback_data": "toggle_warn"}],
            [{"text": "📊 M1 Holder", "callback_data": "toggle_m1"}, {"text": "🚨 M2 Panic", "callback_data": "toggle_m2"}],
            [{"text": "👤 M3 Dev", "callback_data": "toggle_m3"}, {"text": "🔍 M4 Genesis", "callback_data": "toggle_m4"}],
            [{"text": "📝 M5 Metadata", "callback_data": "toggle_m5"}, {"text": "← Back", "callback_data": "ignore"}],
            [{"text": "📋 View Skipped Tokens", "callback_data": "ignore"}],
            [{"text": h30, "callback_data": "set_h30"}, {"text": h40, "callback_data": "set_h40"}, {"text": h50, "callback_data": "set_h50"}],
            [{"text": a0, "callback_data": "set_a0"}, {"text": a3, "callback_data": "set_a3"}, {"text": a6, "callback_data": "set_a6"}],
            [{"text": a12, "callback_data": "set_a12"}, {"text": a24, "callback_data": "set_a24"}],
            [{"text": t0, "callback_data": "set_t0"}, {"text": t5, "callback_data": "set_t5"}, {"text": t10, "callback_data": "set_t10"}]
        ]
    })
}

async fn send_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({
        "chat_id": chat_id,
        "text": build_settings_text(),
        "reply_markup": build_settings_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn update_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64) {
    let url = format!("https://api.telegram.org/bot{}/editMessageText", token);
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "text": build_settings_text(),
        "reply_markup": build_settings_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

// ══════════════════════════════════════════════════════════════════
// Wallet Management Handlers
// ══════════════════════════════════════════════════════════════════

async fn send_wallet_menu(client: &reqwest::Client, token: &str, chat_id: &str) {
    use crate::modules::wallet_manager::WalletStore;

    let store = WalletStore::load();
    let wallet_list = store.display_list();

    // Get balance of active wallet
    let balance_text = if let Some(active) = store.active() {
        let balance = crate::RPC_CLIENT
            .get_balance(&active.pubkey.parse().unwrap_or(*crate::SIGNER_PUBKEY))
            .await
            .unwrap_or(0);
        format!("{:.4} SOL", balance as f64 / 1_000_000_000.0)
    } else {
        "N/A".to_string()
    };

    let msg = format!(
        "💰 <b>Wallet Management</b>\n\n\
         <b>Commands:</b>\n\
         /generate - generate new wallet\n\
         /import_key - import private key via next message\n\
         /select_N - select wallet id N for trading\n\
         /delete_N - delete wallet by id N\n\
         /show_key_N - show private key of wallet N\n\
         /wallets - show this menu\n\n\
         <b>Wallet list:</b>\n\
         {}\
         Balance: <b>{}</b>",
        wallet_list, balance_text
    );
    send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
}

async fn handle_generate_wallet(client: &reqwest::Client, token: &str, chat_id: &str) {
    use crate::modules::wallet_manager::WalletStore;

    let mut store = WalletStore::load();
    let (pubkey, label, idx) = store.generate();
    let msg = format!(
        "✅ <b>New wallet generated!</b>\n\n\
         🔑 <b>Address:</b> <code>{}</code>\n\
         🏷️ <b>Label:</b> {}\n\n\
         Use /select_{} to activate it for trading.\n\
         Use /show_key_{} to see its private key.",
        pubkey, label, idx, idx
    );
    send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
}

async fn handle_import_wallet(client: &reqwest::Client, token: &str, chat_id: &str, private_key: &str) {
    use crate::modules::wallet_manager::WalletStore;

    let mut store = WalletStore::load();
    match store.import(private_key) {
        Ok((pubkey, idx)) => {
            let msg = format!(
                "✅ <b>Wallet imported!</b>\n\n\
                 🔑 <b>Address:</b> <code>{}</code>\n\n\
                 Use /select_{} to activate it for trading.",
                pubkey, idx
            );
            send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
        }
        Err(e) => {
            let msg = format!("❌ <b>Import failed:</b> {}", e);
            send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
        }
    }
}

async fn handle_select_wallet(client: &reqwest::Client, token: &str, chat_id: &str, index: usize) {
    use crate::modules::wallet_manager::{WalletStore, switch_active_wallet_and_restart};

    let mut store = WalletStore::load();
    match store.select(index) {
        Some((pk, pubkey)) => {
            let msg = format!(
                "✅ <b>Selected wallet {}.</b>\n\
                 <code>{}</code>\n\n\
                 🔄 <b>Closing old nonces & Generating new nonces...</b>\n\
                 <i>Please wait ~2-3 minutes for the Solana network to confirm, then send /start</i>",
                index, pubkey
            );
            send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;

            // Give Telegram time to deliver the message
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Switch the active wallet in .env and restart
            if let Err(e) = switch_active_wallet_and_restart(&pk) {
                let err_msg = format!("❌ Failed to switch wallet: {}", e);
                send_simple_msg(client, token, chat_id, &err_msg).await;
            }
        }
        None => {
            send_simple_msg(client, token, chat_id, "❌ Invalid wallet index.").await;
        }
    }
}

async fn handle_delete_wallet(client: &reqwest::Client, token: &str, chat_id: &str, index: usize) {
    use crate::modules::wallet_manager::WalletStore;

    let mut store = WalletStore::load();
    match store.delete(index) {
        Ok(removed) => {
            let msg = format!(
                "🗑️ <b>Wallet {} deleted.</b>\n\
                 <code>{}</code>",
                index, removed.pubkey
            );
            send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
        }
        Err(e) => {
            let msg = format!("❌ {}", e);
            send_simple_msg(client, token, chat_id, &msg).await;
        }
    }
}

async fn handle_show_key(client: &reqwest::Client, token: &str, chat_id: &str, index: usize) {
    use crate::modules::wallet_manager::WalletStore;

    let store = WalletStore::load();
    match store.get(index) {
        Some(entry) => {
            let msg = format!(
                "🔐 <b>Wallet {}. pubkey:</b>\n\
                 <code>{}</code>\n\n\
                 🔑 <b>Private key:</b>\n\
                 <code>{}</code>\n\n\
                 ⚠️ <i>Do NOT share this with anyone!</i>",
                index, entry.pubkey, entry.private_key
            );
            send_simple_msg_with_parse_mode(client, token, chat_id, &msg, "HTML").await;
        }
        None => {
            send_simple_msg(client, token, chat_id, "❌ Invalid wallet index.").await;
        }
    }
}

use crate::*;
use serde_json::{json, Value};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

pub static WARN_ONLY_MODE: AtomicBool = AtomicBool::new(false);
pub static ENABLE_M1_HOLDER: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M2_PANIC: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M3_DEV: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M4_GENESIS: AtomicBool = AtomicBool::new(true);
pub static ENABLE_M5_METADATA: AtomicBool = AtomicBool::new(true);

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
                            if sender_chat_id == chat_id {
                                if text.starts_with("/start") || text.starts_with("/settings") {
                                    send_settings_menu(&client, &token, &chat_id).await;
                                }
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
                                // Answer callback query
                                let answer_url = format!("https://api.telegram.org/bot{}/answerCallbackQuery", token);
                                let _ = client.post(&answer_url).json(&json!({"callback_query_id": callback_id})).send().await;
                            }
                        }
                    }
                }
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn handle_callback(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64, data: &str) {
    match data {
        "toggle_warn" => {
            let current = WARN_ONLY_MODE.load(Ordering::Relaxed);
            WARN_ONLY_MODE.store(!current, Ordering::Relaxed);
        }
        "toggle_m1" => {
            let current = ENABLE_M1_HOLDER.load(Ordering::Relaxed);
            ENABLE_M1_HOLDER.store(!current, Ordering::Relaxed);
        }
        "toggle_m2" => {
            let current = ENABLE_M2_PANIC.load(Ordering::Relaxed);
            ENABLE_M2_PANIC.store(!current, Ordering::Relaxed);
        }
        "toggle_m3" => {
            let current = ENABLE_M3_DEV.load(Ordering::Relaxed);
            ENABLE_M3_DEV.store(!current, Ordering::Relaxed);
        }
        "toggle_m4" => {
            let current = ENABLE_M4_GENESIS.load(Ordering::Relaxed);
            ENABLE_M4_GENESIS.store(!current, Ordering::Relaxed);
        }
        "toggle_m5" => {
            let current = ENABLE_M5_METADATA.load(Ordering::Relaxed);
            ENABLE_M5_METADATA.store(!current, Ordering::Relaxed);
        }
        _ => {}
    }

    update_settings_menu(client, token, chat_id, message_id).await;
}

fn build_keyboard() -> Value {
    let w = if WARN_ONLY_MODE.load(Ordering::Relaxed) { "⚠️ Warn-Only Mode (no block)" } else { "❌ Warn-Only Mode (no block)" };
    let m1 = if ENABLE_M1_HOLDER.load(Ordering::Relaxed) { "✅ M1: Holder Analyzer" } else { "❌ M1: Holder Analyzer" };
    let m2 = if ENABLE_M2_PANIC.load(Ordering::Relaxed) { "✅ M2: Panic-Sell Monitor" } else { "❌ M2: Panic-Sell Monitor" };
    let m3 = if ENABLE_M3_DEV.load(Ordering::Relaxed) { "✅ M3: Dev Wallet Profiler" } else { "❌ M3: Dev Wallet Profiler" };
    let m4 = if ENABLE_M4_GENESIS.load(Ordering::Relaxed) { "✅ M4: Genesis Detector" } else { "❌ M4: Genesis Detector" };
    let m5 = if ENABLE_M5_METADATA.load(Ordering::Relaxed) { "✅ M5: Metadata Checker" } else { "❌ M5: Metadata Checker" };

    json!({
        "inline_keyboard": [
            [{"text": "🛡️ Anti-Rug Intelligence Settings", "callback_data": "ignore"}],
            [{"text": "Tap to toggle each module:", "callback_data": "ignore"}],
            [{"text": "🔄 Anti-Rug Master Switch", "callback_data": "ignore"}],
            [{"text": w, "callback_data": "toggle_warn"}],
            [{"text": m1, "callback_data": "toggle_m1"}],
            [{"text": m2, "callback_data": "toggle_m2"}],
            [{"text": m3, "callback_data": "toggle_m3"}],
            [{"text": m4, "callback_data": "toggle_m4"}],
            [{"text": m5, "callback_data": "toggle_m5"}],
            [{"text": "🔙 Back to Trading", "callback_data": "ignore"}]
        ]
    })
}

async fn send_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str) {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let payload = json!({
        "chat_id": chat_id,
        "text": "⚙️ **Anti-Rug Intelligence Settings**",
        "parse_mode": "Markdown",
        "reply_markup": build_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

async fn update_settings_menu(client: &reqwest::Client, token: &str, chat_id: &str, message_id: i64) {
    let url = format!("https://api.telegram.org/bot{}/editMessageReplyMarkup", token);
    let payload = json!({
        "chat_id": chat_id,
        "message_id": message_id,
        "reply_markup": build_keyboard()
    });
    let _ = client.post(&url).json(&payload).send().await;
}

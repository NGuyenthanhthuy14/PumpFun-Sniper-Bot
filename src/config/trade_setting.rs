use crate::*;
use once_cell::sync::Lazy;

pub static STOP_LOSS: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.stop_loss / 100.0);
pub static TP_TRAILING: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.tp_trailing);
pub static TRAILING_STOP: Lazy<f64> = Lazy::new(|| CONFIG.sell_setting.trailing_stop);
pub static NO_ACTIVITY_TIME: Lazy<u64> = Lazy::new(|| CONFIG.sell_setting.no_activity_time);
pub static STOP_NO_ACTIVITY_TOKEN_MONITORING: Lazy<bool> =
    Lazy::new(|| CONFIG.sell_setting.no_activity_time > 0);

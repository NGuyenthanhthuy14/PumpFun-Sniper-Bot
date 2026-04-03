// use crate::*;
// use magic_crypt::{MagicCryptTrait, new_magic_crypt};
// use std::collections::HashMap;
use std::time::Duration;
use std::time::{Instant, SystemTime};

pub fn get_system_time_from_instant(instant: Instant) -> SystemTime {
    // Get current system time
    let now_system = SystemTime::now();
    
    // Get current instant
    let now_instant = Instant::now();
    
    // Calculate the duration since the reference instant
    let elapsed = now_instant.duration_since(instant);
    
    // Apply the elapsed duration to system time
    let system_time = now_system - elapsed;
    system_time
}

pub fn format_elapsed_time(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    let nanos = elapsed.subsec_nanos();

    let seconds = secs;
    let millis = nanos / 1_000_000;
    let micros = (nanos % 1_000_000) / 1_000;

    let mut parts = Vec::new();

    if seconds > 0 {
        parts.push(format!("{}s", seconds));
    }
    if millis > 0 {
        parts.push(format!("{}ms", millis));
    }
    if micros > 0 && millis == 0 {
        parts.push(format!("{}µs", micros));
    }

    if parts.is_empty() {
        parts.push("0µs".to_string());
    }

    parts.join(" : ")
}
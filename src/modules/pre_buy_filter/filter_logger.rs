/// Phase 2 — Filter Audit Logger
///
/// Records EVERY filter decision to CSV for later analysis and calibration.
/// This is critical for Phase E (paper-trading) where we need to measure
/// filter effectiveness: true positives, false positives, accuracy.
///
/// Thread-safe: uses Arc<Mutex<Writer>> so multiple async tasks can log.
/// Auto-rotates files daily if needed.

use crate::*;
use once_cell::sync::Lazy;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

/// Global filter logger instance.
/// Initialized on first use; logs to `filter_audit_{date}.csv`.
static FILTER_LOGGER: Lazy<Mutex<FilterLogger>> = Lazy::new(|| {
    Mutex::new(FilterLogger::new())
});

struct FilterLogger {
    /// Current CSV writer (if logging is enabled)
    writer: Option<csv::Writer<std::fs::File>>,
    /// Date string for the current log file
    current_date: String,
}

impl FilterLogger {
    fn new() -> Self {
        if !*FILTER_LOG_ENABLED {
            return Self {
                writer: None,
                current_date: String::new(),
            };
        }

        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let writer = Self::create_writer(&today);

        Self {
            writer,
            current_date: today,
        }
    }

    fn create_writer(date: &str) -> Option<csv::Writer<std::fs::File>> {
        let log_dir = FILTER_LOG_DIR.as_str();

        // Create log directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(log_dir) {
            error!("[FILTER_LOG] Failed to create log dir '{}': {}", log_dir, e);
            return None;
        }

        let file_path = format!("{}/filter_audit_{}.csv", log_dir, date);
        let file_exists = std::path::Path::new(&file_path).exists();

        match OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
        {
            Ok(file) => {
                let mut writer = csv::Writer::from_writer(file);

                // Write header if file is new
                if !file_exists {
                    let _ = writer.write_record(&[
                        "timestamp",
                        "mint",
                        "creator",
                        "token_name",
                        "token_symbol",
                        "module_name",
                        "passed",
                        "risk_score",
                        "reason",
                        "total_risk_score",
                        "final_decision",
                        "buy_multiplier",
                    ]);
                    let _ = writer.flush();
                }

                info!("[FILTER_LOG] Logging to: {}", file_path);
                Some(writer)
            }
            Err(e) => {
                error!("[FILTER_LOG] Failed to open '{}': {}", file_path, e);
                None
            }
        }
    }

    /// Check if we need to rotate to a new daily file.
    fn maybe_rotate(&mut self) {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if today != self.current_date {
            info!(
                "[FILTER_LOG] Day changed {} → {} — rotating log file",
                self.current_date, today
            );
            // Flush old writer
            if let Some(ref mut w) = self.writer {
                let _ = w.flush();
            }
            self.writer = Self::create_writer(&today);
            self.current_date = today;
        }
    }

    /// Write a single audit record.
    fn write_record(&mut self, record: &FilterAuditRecord) {
        self.maybe_rotate();

        if let Some(ref mut writer) = self.writer {
            let _ = writer.serialize(record);
            let _ = writer.flush();
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// Public API
// ══════════════════════════════════════════════════════════════════════

/// Log an aggregated filter result to the CSV audit trail.
/// Called once per token evaluation, writing one row per filter module.
pub fn log_filter_result(ctx: &FilterContext, aggregated: &AggregatedFilterResult) {
    if !*FILTER_LOG_ENABLED {
        return;
    }

    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    let final_decision = if aggregated.should_buy {
        "BUY".to_string()
    } else {
        "SKIP".to_string()
    };

    let mut logger = match FILTER_LOGGER.lock() {
        Ok(l) => l,
        Err(e) => {
            error!("[FILTER_LOG] Lock poisoned: {}", e);
            return;
        }
    };

    for result in &aggregated.results {
        let record = FilterAuditRecord {
            timestamp: timestamp.clone(),
            mint: ctx.mint.to_string(),
            creator: ctx.creator.to_string(),
            token_name: ctx.name.clone(),
            token_symbol: ctx.symbol.clone(),
            module_name: result.module_name.clone(),
            passed: result.passed,
            risk_score: result.risk_score,
            reason: result.reason.clone(),
            total_risk_score: aggregated.total_risk_score,
            final_decision: final_decision.clone(),
            buy_multiplier: aggregated.buy_amount_multiplier,
        };

        logger.write_record(&record);
    }
}

/// Phase 2 — Social/Metadata Verification (Nâng cao)
///
/// Legitimate tokens typically have proper Metaplex metadata:
///   - Non-empty URI pointing to a JSON file (IPFS/Arweave)
///   - JSON contains: name, symbol, description, image
///   - Social links: twitter, telegram, website
///
/// This module performs two levels of checks:
///
///   Level 1 (Synchronous, zero-latency):
///     - Validate name/symbol from MintEvent
///     - Check URI is non-empty
///     - Anti-spam pattern detection
///     - Duplicate name detection
///
///   Level 2 (Async, optional):
///     - HTTP GET the URI, parse JSON
///     - Check for image, description, social links
///     - Score based on metadata completeness

use crate::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;
use tokio::time::timeout;

const MODULE_NAME: &str = "METADATA_CHECKER";

/// Track recently seen token names for duplicate detection.
/// Key: lowercase name → Value: list of mints with that name
static NAME_TRACKER: Lazy<DashMap<String, Vec<(Pubkey, std::time::Instant)>>> =
    Lazy::new(DashMap::new);

/// How long to remember names for duplicate detection (5 minutes).
const NAME_TRACKER_TTL_SECS: u64 = 300;

// ══════════════════════════════════════════════════════════════════════
// Metaplex JSON metadata schema
// ══════════════════════════════════════════════════════════════════════

/// Partial Metaplex metadata JSON schema.
/// We only parse fields we need for verification.
#[derive(Debug, serde::Deserialize, Default)]
struct MetaplexMetadata {
    #[serde(default)]
    name: String,
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    image: String,
    #[serde(default)]
    external_url: String,
    // Social links can be in various locations
    #[serde(default)]
    twitter: String,
    #[serde(default)]
    telegram: String,
    #[serde(default)]
    website: String,
}

// ══════════════════════════════════════════════════════════════════════
// Public API
// ══════════════════════════════════════════════════════════════════════

/// Check token metadata — combines sync + async checks.
///
/// Level 1 (sync): name/symbol validation, spam detection, duplicates
/// Level 2 (async): URI content fetch and parse (if enabled)
pub async fn check_metadata(ctx: &FilterContext) -> FilterResult {
    if !*METADATA_CHECKER_ENABLED {
        return FilterResult::pass(MODULE_NAME);
    }

    let mut risk_score: f64 = 0.0;
    let mut warnings: Vec<String> = Vec::new();

    // If ALL metadata fields are empty, this is a bundle match context where
    // name/symbol/uri are not available. Skip metadata checks entirely to
    // avoid false positives from missing data.
    if ctx.name.trim().is_empty() && ctx.symbol.trim().is_empty() && ctx.uri.trim().is_empty() {
        return FilterResult::pass(MODULE_NAME);
    }

    // ══════════════════════════════════════════════════════════
    // Level 1: Synchronous checks (zero-latency)
    // ══════════════════════════════════════════════════════════

    // Check 1: URI validation
    let uri_trimmed = ctx.uri.trim();
    if uri_trimmed.is_empty() {
        if *REQUIRE_METADATA_URI {
            match METADATA_EMPTY_ACTION.as_str() {
                "skip" => {
                    info!(
                        "[{}] REJECT: {} | No metadata URI — likely scam",
                        MODULE_NAME, ctx.mint
                    );
                    return FilterResult::fail(
                        MODULE_NAME,
                        "No metadata URI — likely scam/rug".to_string(),
                        30.0,
                    );
                }
                "warn" => {
                    risk_score += 20.0;
                    warnings.push("No metadata URI".to_string());
                }
                _ => {} // "allow"
            }
        }
    }

    // Check 2: Name validation
    let name_trimmed = ctx.name.trim();
    if name_trimmed.len() < *MIN_NAME_LENGTH {
        risk_score += 15.0;
        warnings.push(format!(
            "Name too short: '{}' (len={} < {})",
            name_trimmed,
            name_trimmed.len(),
            *MIN_NAME_LENGTH
        ));
    }

    // Check 3: Symbol validation
    let symbol_trimmed = ctx.symbol.trim();
    if symbol_trimmed.len() < *MIN_SYMBOL_LENGTH {
        risk_score += 15.0;
        warnings.push(format!(
            "Symbol too short: '{}' (len={} < {})",
            symbol_trimmed,
            symbol_trimmed.len(),
            *MIN_SYMBOL_LENGTH
        ));
    }

    // Check 4: Spam pattern detection
    if name_trimmed.len() >= 3 {
        if let Some(spam_reason) = detect_spam_pattern(name_trimmed) {
            risk_score += 10.0;
            warnings.push(format!("Suspicious name: {}", spam_reason));
        }
    }

    if symbol_trimmed.len() >= 2 {
        if let Some(spam_reason) = detect_spam_pattern(symbol_trimmed) {
            risk_score += 5.0;
            warnings.push(format!("Suspicious symbol: {}", spam_reason));
        }
    }

    // Check 5: Duplicate name detection
    let name_lower = name_trimmed.to_lowercase();
    if !name_lower.is_empty() {
        let is_duplicate = check_and_register_name(&name_lower, ctx.mint);
        if is_duplicate {
            risk_score += 20.0;
            warnings.push(format!(
                "Duplicate name '{}' — possible copycat",
                name_trimmed
            ));
        }
    }

    // ══════════════════════════════════════════════════════════
    // Level 2: Async URI content fetch (if enabled and URI exists)
    // ══════════════════════════════════════════════════════════

    if *FETCH_URI_CONTENT && !uri_trimmed.is_empty() {
        let uri_timeout = Duration::from_millis(*URI_TIMEOUT_MS);

        match timeout(uri_timeout, fetch_and_score_metadata(uri_trimmed)).await {
            Ok((uri_risk_delta, uri_warnings)) => {
                risk_score += uri_risk_delta;
                warnings.extend(uri_warnings);
            }
            Err(_) => {
                // URI fetch timed out — mild warning
                risk_score += 5.0;
                warnings.push(format!("URI fetch timeout ({}ms)", *URI_TIMEOUT_MS));
            }
        }
    }

    // ══════════════════════════════════════════════════════════
    // Final result
    // ══════════════════════════════════════════════════════════

    // Metadata hard-reject threshold: when action="skip", if cumulative metadata
    // risk hits 25+, hard-reject. This catches: no URI (30) or short name+symbol (30).
    // Threshold is proportional to avoid being too aggressive with minor issues.
    let metadata_reject_threshold = 25.0;
    if risk_score >= metadata_reject_threshold && METADATA_EMPTY_ACTION.as_str() == "skip" {
        let reason = warnings.join("; ");
        info!(
            "[{}] REJECT: {} | name='{}' sym='{}' | risk={:.0} | {}",
            MODULE_NAME, ctx.mint, name_trimmed, symbol_trimmed, risk_score, reason
        );
        FilterResult::fail(MODULE_NAME, reason, risk_score)
    } else if risk_score > 0.0 {
        let reason = warnings.join("; ");
        FilterResult::warn(MODULE_NAME, reason, risk_score)
    } else {
        FilterResult::pass(MODULE_NAME)
    }
}

// ══════════════════════════════════════════════════════════════════════
// Level 2: URI content analysis
// ══════════════════════════════════════════════════════════════════════

/// Fetch URI content, parse as Metaplex JSON, score completeness.
/// Returns (risk_delta, warnings). Negative risk = reduces total risk.
async fn fetch_and_score_metadata(uri: &str) -> (f64, Vec<String>) {
    let mut risk_delta: f64 = 0.0;
    let mut warnings: Vec<String> = Vec::new();

    // Build HTTP client for this request
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(*URI_TIMEOUT_MS))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!("HTTP client error: {}", e));
            return (5.0, warnings);
        }
    };

    // Fetch URI content
    let response = match client.get(uri).send().await {
        Ok(resp) => resp,
        Err(e) => {
            risk_delta += 10.0;
            warnings.push(format!("URI unreachable: {}", e));
            return (risk_delta, warnings);
        }
    };

    if !response.status().is_success() {
        risk_delta += 10.0;
        warnings.push(format!("URI returned HTTP {}", response.status()));
        return (risk_delta, warnings);
    }

    // Parse response body as JSON
    let body = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            risk_delta += 5.0;
            warnings.push(format!("Failed to read URI body: {}", e));
            return (risk_delta, warnings);
        }
    };

    let metadata: MetaplexMetadata = match serde_json::from_str(&body) {
        Ok(m) => m,
        Err(e) => {
            risk_delta += 15.0;
            warnings.push(format!("URI JSON parse failed: {}", e));
            return (risk_delta, warnings);
        }
    };

    // Score based on metadata completeness
    // Having proper metadata REDUCES risk (negative delta)

    if !metadata.image.trim().is_empty() {
        risk_delta -= 5.0; // Has image
    } else {
        risk_delta += 5.0;
        warnings.push("No image in metadata".to_string());
    }

    if metadata.description.trim().len() > 20 {
        risk_delta -= 5.0; // Has meaningful description
    }

    if !metadata.twitter.trim().is_empty() || body.to_lowercase().contains("twitter.com") || body.to_lowercase().contains("x.com") {
        risk_delta -= 10.0; // Has Twitter — strong positive signal
    }

    if !metadata.telegram.trim().is_empty() || body.to_lowercase().contains("t.me") {
        risk_delta -= 5.0; // Has Telegram
    }

    if !metadata.website.trim().is_empty()
        || !metadata.external_url.trim().is_empty()
        || body.to_lowercase().contains("website")
    {
        risk_delta -= 5.0; // Has website
    }

    // Clamp: don't let good metadata reduce below -15
    risk_delta = risk_delta.max(-15.0);

    (risk_delta, warnings)
}

// ══════════════════════════════════════════════════════════════════════
// Spam and duplicate detection
// ══════════════════════════════════════════════════════════════════════

/// Detect spam patterns in a name or symbol.
/// Returns Some(reason) if spam detected, None otherwise.
fn detect_spam_pattern(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();

    // All same character repeated (e.g., "AAAA", "xxxx")
    if chars.len() >= 3 && chars.iter().all(|c| *c == chars[0]) {
        return Some(format!(
            "All same character repeated: '{}'",
            chars[0]
        ));
    }

    // High ratio of non-alphanumeric characters (excluding spaces)
    let non_alnum = chars
        .iter()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count();
    if chars.len() >= 3 && non_alnum as f64 / chars.len() as f64 > 0.5 {
        return Some(format!(
            "High special char ratio: {}/{}",
            non_alnum,
            chars.len()
        ));
    }

    // Numeric-only names (e.g., "123456")
    if chars.len() >= 3 && chars.iter().all(|c| c.is_ascii_digit()) {
        return Some("Numeric-only name".to_string());
    }

    None
}

/// Check if a name was recently used by another token and register the current one.
/// Returns true if this is a duplicate.
fn check_and_register_name(name_lower: &str, mint: Pubkey) -> bool {
    let now = std::time::Instant::now();
    let ttl = Duration::from_secs(NAME_TRACKER_TTL_SECS);

    let mut is_duplicate = false;

    NAME_TRACKER
        .entry(name_lower.to_string())
        .and_modify(|entries| {
            // Remove expired entries
            entries.retain(|(_, ts)| ts.elapsed() < ttl);

            // Check if any remaining entries exist (that aren't this mint)
            if entries.iter().any(|(m, _)| *m != mint) {
                is_duplicate = true;
            }

            // Register this mint
            entries.push((mint, now));
        })
        .or_insert_with(|| vec![(mint, now)]);

    is_duplicate
}

/// Clean up expired name tracker entries. Called periodically.
pub fn metadata_name_tracker_cleanup() {
    let ttl = Duration::from_secs(NAME_TRACKER_TTL_SECS);
    NAME_TRACKER.retain(|_, entries| {
        entries.retain(|(_, ts)| ts.elapsed() < ttl);
        !entries.is_empty()
    });
}

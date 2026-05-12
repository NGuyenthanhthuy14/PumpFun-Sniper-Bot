/// Phase 2 — Pre-Buy Filter Types
///
/// Shared data structures for the Anti-Rug filter pipeline.
/// All filter modules consume a `FilterContext` and produce a `FilterResult`.
/// The aggregator combines results into `AggregatedFilterResult`.
///
/// Design principles:
///   - FilterContext carries ALL data a filter might need (avoid global access)
///   - FilterResult includes machine-readable fields for CSV logging
///   - AggregatedFilterResult supports weighted scoring

use solana_sdk::pubkey::Pubkey;
use std::time::Instant;

// ══════════════════════════════════════════════════════════════════════
// FilterContext — all data available for filter evaluation
// ══════════════════════════════════════════════════════════════════════

/// Everything a filter module needs to evaluate a token.
/// Built once per mint event and passed to all modules.
#[derive(Debug, Clone)]
pub struct FilterContext {
    /// Token mint address
    pub mint: Pubkey,
    /// Dev/creator wallet address
    pub creator: Pubkey,
    /// Token name from MintEvent
    pub name: String,
    /// Token symbol from MintEvent
    pub symbol: String,
    /// Metadata URI from MintEvent
    pub uri: String,
    /// Total token supply (raw, 6 decimals)
    pub total_supply: u64,
    /// Slot in which this token was minted (from gRPC SubscribeUpdate)
    pub creation_slot: u64,
    /// Timestamp when the filter context was created
    pub created_at: Instant,
    /// Initial token price at mint
    pub initial_price: f64,
}

impl FilterContext {
    /// Create a new FilterContext from mint event data.
    pub fn new(
        mint: Pubkey,
        creator: Pubkey,
        name: String,
        symbol: String,
        uri: String,
        total_supply: u64,
        creation_slot: u64,
        initial_price: f64,
    ) -> Self {
        Self {
            mint,
            creator,
            name,
            symbol,
            uri,
            total_supply,
            creation_slot,
            created_at: Instant::now(),
            initial_price,
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// FilterResult — output from a single filter module
// ══════════════════════════════════════════════════════════════════════

/// Result produced by a single filter module.
#[derive(Debug, Clone)]
pub struct FilterResult {
    /// Did this filter pass (true) or hard-fail (false)?
    pub passed: bool,
    /// Human-readable reason (logged & sent to alerts)
    pub reason: String,
    /// Numeric risk contribution (0.0 = no risk, higher = worse)
    pub risk_score: f64,
    /// Name of the filter module that produced this result
    pub module_name: String,
}

impl FilterResult {
    /// Token passed this filter with zero risk.
    pub fn pass(module: &str) -> Self {
        Self {
            passed: true,
            reason: "OK".to_string(),
            risk_score: 0.0,
            module_name: module.to_string(),
        }
    }

    /// Token FAILED this filter — will be rejected regardless of other filters.
    pub fn fail(module: &str, reason: impl Into<String>, risk: f64) -> Self {
        Self {
            passed: false,
            reason: reason.into(),
            risk_score: risk,
            module_name: module.to_string(),
        }
    }

    /// Token passed but with elevated risk score (contributes to total risk).
    pub fn warn(module: &str, reason: impl Into<String>, risk: f64) -> Self {
        Self {
            passed: true,
            reason: reason.into(),
            risk_score: risk,
            module_name: module.to_string(),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// AggregatedFilterResult — combined output from all modules
// ══════════════════════════════════════════════════════════════════════

/// Combined result from all filter modules.
#[derive(Debug, Clone)]
pub struct AggregatedFilterResult {
    /// Final decision: should the bot buy this token?
    pub should_buy: bool,
    /// Sum of all risk scores
    pub total_risk_score: f64,
    /// Suggested buy amount multiplier (1.0 = normal, <1.0 = reduced)
    /// Allows dynamic position sizing based on risk.
    pub buy_amount_multiplier: f64,
    /// Individual results from each module
    pub results: Vec<FilterResult>,
}

impl AggregatedFilterResult {
    /// Build a human-readable summary of why a token was rejected or flagged.
    pub fn rejection_summary(&self) -> String {
        self.results
            .iter()
            .filter(|r| !r.passed || r.risk_score > 0.0)
            .map(|r| {
                format!(
                    "[{}] {} (risk: {:.1})",
                    r.module_name, r.reason, r.risk_score
                )
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

// ══════════════════════════════════════════════════════════════════════
// FilterAuditRecord — for CSV logging
// ══════════════════════════════════════════════════════════════════════

/// A single row in the filter audit CSV log.
/// Records every filter decision for later analysis and calibration.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FilterAuditRecord {
    /// ISO-8601 timestamp
    pub timestamp: String,
    /// Token mint address (base58)
    pub mint: String,
    /// Dev/creator address (base58)
    pub creator: String,
    /// Token name
    pub token_name: String,
    /// Token symbol
    pub token_symbol: String,
    /// Which filter module produced this record
    pub module_name: String,
    /// Did this module pass?
    pub passed: bool,
    /// Risk score from this module
    pub risk_score: f64,
    /// Reason string
    pub reason: String,
    /// Total risk score across all modules
    pub total_risk_score: f64,
    /// Final aggregated decision
    pub final_decision: String,
    /// Suggested buy multiplier
    pub buy_multiplier: f64,
}

// ══════════════════════════════════════════════════════════════════════
// Genesis buy tracking data
// ══════════════════════════════════════════════════════════════════════

/// Tracks a single buy event for genesis analysis.
#[derive(Debug, Clone)]
pub struct GenesisBuyRecord {
    /// Buyer wallet address
    pub buyer: Pubkey,
    /// Token amount purchased (raw, 6 decimals)
    pub token_amount: u64,
    /// SOL amount spent (lamports)
    pub sol_amount: u64,
    /// Slot in which this buy occurred
    pub slot: u64,
}

/// Aggregated genesis data for a single mint.
#[derive(Debug, Clone)]
pub struct GenesisTrackingData {
    /// Slot where the token was created
    pub creation_slot: u64,
    /// Total token supply
    pub total_supply: u64,
    /// Creator/dev address
    pub creator: Pubkey,
    /// When tracking started
    pub created_at: Instant,
    /// All recorded buy events in genesis window
    pub buy_records: Vec<GenesisBuyRecord>,
}

impl GenesisTrackingData {
    pub fn new(creation_slot: u64, total_supply: u64, creator: Pubkey) -> Self {
        Self {
            creation_slot,
            total_supply,
            creator,
            created_at: Instant::now(),
            buy_records: Vec::new(),
        }
    }

    /// Total tokens bought across all genesis buys.
    pub fn total_tokens_bought(&self) -> u64 {
        self.buy_records.iter().map(|r| r.token_amount).sum()
    }

    /// Percentage of total supply bought in genesis window.
    pub fn genesis_buy_percent(&self) -> f64 {
        if self.total_supply == 0 {
            return 0.0;
        }
        (self.total_tokens_bought() as f64 / self.total_supply as f64) * 100.0
    }

    /// Number of unique buyer wallets.
    pub fn unique_buyer_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for record in &self.buy_records {
            seen.insert(record.buyer);
        }
        seen.len()
    }

    /// Number of unique buyer wallets excluding the creator.
    pub fn non_creator_buyer_count(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for record in &self.buy_records {
            if record.buyer != self.creator {
                seen.insert(record.buyer);
            }
        }
        seen.len()
    }

    /// Check if a buy is within the genesis analysis window.
    pub fn is_within_genesis_window(&self, buy_slot: u64, max_slots: u64) -> bool {
        buy_slot <= self.creation_slot + max_slots
    }

    /// Get the single largest buyer's percentage of supply.
    pub fn largest_single_buyer_percent(&self) -> f64 {
        if self.total_supply == 0 {
            return 0.0;
        }
        let mut per_buyer: std::collections::HashMap<Pubkey, u64> =
            std::collections::HashMap::new();
        for record in &self.buy_records {
            *per_buyer.entry(record.buyer).or_insert(0) += record.token_amount;
        }
        let max_amount = per_buyer.values().copied().max().unwrap_or(0);
        (max_amount as f64 / self.total_supply as f64) * 100.0
    }
}

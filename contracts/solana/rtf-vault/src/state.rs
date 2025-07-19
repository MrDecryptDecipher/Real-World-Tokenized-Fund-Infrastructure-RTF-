use anchor_lang::prelude::*;

/// Advanced RTF Vault Account with Multi-Chain State Tracking
#[account]
pub struct VaultAccount {
    /// Core vault configuration
    pub authority: Pubkey,
    pub config: VaultConfig,

    /// Financial state
    pub total_assets: u64,
    pub total_liabilities: u64,
    pub nav_per_share: u64,
    pub last_nav_update: i64,
    pub epoch: u64,
    pub status: VaultStatus,

    /// Multi-tranche system (up to 5 tranches)
    pub tranches: [Tranche; 5],
    pub active_tranche_count: u8,

    /// Advanced redemption queue with MEV protection
    pub redemption_queue: RedemptionQueue,

    /// Performance and risk metrics
    pub performance_metrics: PerformanceMetrics,
    pub risk_metrics: RiskMetrics,

    /// Cross-chain state tracking
    pub cross_chain_state: CrossChainState,

    /// zkNAV and integrity tracking
    pub zk_nav_state: ZkNavState,
    pub drift_ledger: DriftLedger,

    /// Emergency and governance state
    pub emergency_state: EmergencyState,
    pub governance_state: GovernanceState,

    /// Fund exposure and isolation
    pub fund_origin_hash: [u8; 32],
    pub exposure_graph: ExposureGraph,

    /// LLM agent integrity
    pub llm_state: LlmAgentState,

    /// ESG and compliance
    pub esg_state: EsgState,
    pub compliance_state: ComplianceState,

    /// PRD: Legal document hash for anchoring
    pub legal_doc_hash: [u8; 32],

    /// Vault metadata
    pub bump: u8,
    pub reserved: [u8; 64], // Reserved for future upgrades
}

/// Comprehensive vault configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct VaultConfig {
    pub underlying_mint: Pubkey,
    pub oracle_authority: Pubkey,
    pub emergency_pause_authority: Pubkey,
    pub operator: Pubkey,

    /// Capacity and utilization limits
    pub max_capacity: u64,
    pub max_utilization: u64, // Basis points (10000 = 100%)

    /// NAV and drift parameters
    pub max_nav_drift: u64, // Basis points
    pub nav_update_frequency: u64, // Seconds

    /// Redemption queue configuration
    pub max_redemption_queue_size: u64,
    pub redemption_processing_window: u64, // Seconds
    pub mev_protection_delay: u64, // Slots
    pub batch_size: u8,

    /// Fee structure
    pub management_fee: u16, // Basis points
    pub performance_fee: u16, // Basis points
    pub redemption_fee: u16, // Basis points

    /// Post-quantum security
    pub enable_post_quantum: bool,
    pub dilithium_public_key: [u8; 64],

    /// Cross-chain configuration
    pub ethereum_contract: [u8; 20],
    pub starknet_contract: [u8; 32],
    pub bitcoin_anchor_address: [u8; 32],
}

/// Enhanced tranche structure with waterfall logic
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct Tranche {
    pub tranche_type: TrancheType,
    pub mint: Pubkey,
    pub total_supply: u64,
    pub nav_per_share: u64,
    pub fee_rate: u16, // Basis points
    pub min_deposit: u64,
    pub max_deposit: u64,
    pub lock_period: u64, // Seconds
    pub yield_rate: u64, // Basis points
    pub last_yield_update: i64,
    pub waterfall_priority: u8,
    pub protection_level: u8, // 0-100 (100 = fully protected)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum TrancheType {
    Senior,
    Mezzanine,
    Junior,
    LP,
    Equity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum VaultStatus {
    Active,
    Paused,
    Emergency,
    Deprecated,
}

/// Additional state structures and events for the RTF Vault

/// Cross-chain state synchronization
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct CrossChainState {
    pub ethereum_root: [u8; 32],
    pub bitcoin_anchor: [u8; 32],
    pub starknet_proof: [u8; 32],
    pub last_sync_timestamp: i64,
    pub sync_status: SyncStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Failed,
}

/// zkNAV state tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ZkNavState {
    pub current_proof: [u8; 32],
    pub last_computation: i64,
    pub computation_frequency: u64,
    pub proof_verification_count: u64,
    pub failed_verifications: u64,
}

/// Drift ledger for 100-epoch tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct DriftLedger {
    pub epoch_drifts: [u64; 100],
    pub current_index: u8,
    pub max_drift_threshold: u64,
    pub consecutive_violations: u8,
}

/// Emergency state management
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct EmergencyState {
    pub is_emergency: bool,
    pub emergency_reason: EmergencyReason,
    pub triggered_by: Pubkey,
    pub triggered_at: i64,
    pub recovery_deadline: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum EmergencyReason {
    None,
    MarketCrash,
    OracleFailure,
    SecurityBreach,
    RegulatoryAction,
    TechnicalFailure,
    ExcessiveDrift,
}

/// Governance state tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct GovernanceState {
    pub active_proposals: u64,
    pub last_proposal_timestamp: i64,
    pub total_voting_power: u64,
    pub quorum_threshold: u64,
    pub proposal_bond_amount: u64,
}

/// Fund exposure graph for circular dependency detection
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ExposureGraph {
    #[max_len(10)]
    pub connected_funds: Vec<Pubkey>,
    #[max_len(10)]
    pub exposure_weights: Vec<u64>, // Basis points
    pub total_exposure: u64,
    pub circular_dependency_detected: bool,
}

/// LLM agent integrity state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct LlmAgentState {
    pub last_output_hash: [u8; 32],
    pub output_count: u64,
    pub deviation_score: u64,
    pub confidence_threshold: u8,
    pub last_simulation_timestamp: i64,
}

/// ESG compliance state
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct EsgState {
    pub carbon_score: u64,
    pub sustainability_rating: u8,
    pub esg_tokens_required: bool,
    pub last_esg_verification: i64,
    pub esg_override_locked: bool,
}

/// Compliance state tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ComplianceState {
    pub kyc_provider_count: u8,
    pub jurisdictional_restrictions: [bool; 32], // Support for 32 jurisdictions
    pub last_compliance_check: i64,
    pub violation_count: u64,
    pub compliance_score: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RedemptionRequest {
    pub user: Pubkey,
    pub tranche_index: u8,
    pub shares_amount: u64,
    pub expected_assets: u64,
    pub request_timestamp: i64,
    pub processing_slot: u64,
    pub status: RedemptionStatus,
    pub commitment_hash: [u8; 32],
    pub bonding_amount: u64,      // PRD: Dynamic bonding under pool stress
    pub reveal_deadline: i64,     // PRD: Commit-reveal scheme deadline
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum RedemptionStatus {
    Pending,
    Committed,    // PRD: Commit phase of commit-reveal scheme
    Revealed,     // PRD: Reveal phase of commit-reveal scheme
    Processing,
    Completed,
    Cancelled,
    Failed,
}

/// PRD: Advanced redemption queue with MEV protection
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RedemptionQueue {
    pub head: u64,
    pub tail: u64,
    pub total_pending: u64,
    pub max_queue_size: u64,
    pub processing_window: u64,
    pub mev_protection_delay: u64,
    pub batch_size: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct TrancheConfig {
    pub tranche_type: crate::TrancheType,
    pub mint: Pubkey,
    pub fee_rate: u16,
    pub min_deposit: u64,
    pub max_deposit: u64,
    pub lock_period: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct NAVData {
    pub nav_per_share: u64,
    pub total_assets: u64,
    pub total_liabilities: u64,
    pub timestamp: i64,
    #[max_len(5)]
    pub tranche_navs: Vec<u64>,
    pub oracle_signature: [u8; 64],
    pub post_quantum_signature: [u8; 128],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct DepositRecord {
    pub user: Pubkey,
    pub tranche_index: u8,
    pub amount: u64,
    pub shares_minted: u64,
    pub nav_per_share: u64,
    pub timestamp: i64,
    pub epoch: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct WithdrawalRecord {
    pub user: Pubkey,
    pub tranche_index: u8,
    pub shares_burned: u64,
    pub assets_returned: u64,
    pub nav_per_share: u64,
    pub fee_paid: u64,
    pub timestamp: i64,
    pub epoch: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ComplianceData {
    pub kyc_verified: bool,
    pub jurisdiction: [u8; 2], // ISO country code
    pub accredited_investor: bool,
    pub risk_score: u8,
    pub last_verification: i64,
    pub verification_provider: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RiskMetrics {
    pub var_95: u64,        // Value at Risk 95%
    pub var_99: u64,        // Value at Risk 99%
    pub volatility: u64,    // Annualized volatility
    pub sharpe_ratio: i64,  // Sharpe ratio (can be negative)
    pub max_drawdown: u64,  // Maximum drawdown
    pub beta: i64,          // Beta to market (can be negative)
    pub last_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PerformanceMetrics {
    pub total_return: i64,      // Total return since inception
    pub annualized_return: i64, // Annualized return
    pub monthly_returns: [i64; 12], // Last 12 months
    pub benchmark_return: i64,   // Benchmark comparison
    pub tracking_error: u64,    // Tracking error vs benchmark
    pub information_ratio: i64, // Information ratio
    pub last_update: i64,
}

// Events
#[event]
pub struct VaultInitialized {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub config: crate::VaultConfig,
    pub timestamp: i64,
}

#[event]
pub struct DepositMade {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub tranche_index: u8,
    pub amount: u64,
    pub shares_minted: u64,
    pub record: DepositRecord,
}

#[event]
pub struct RedemptionRequested {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub tranche_index: u8,
    pub shares_amount: u64,
    pub expected_assets: u64,
    pub queue_position: u64,
    pub processing_slot: u64,
}

#[event]
pub struct RedemptionsProcessed {
    pub vault: Pubkey,
    pub processed_count: u8,
    pub total_assets_redeemed: u64,
    pub remaining_queue_size: u64,
    pub timestamp: i64,
}

#[event]
pub struct NAVUpdated {
    pub vault: Pubkey,
    pub old_nav: u64,
    pub new_nav: u64,
    pub total_assets: u64,
    pub total_liabilities: u64,
    pub timestamp: i64,
    pub oracle: Pubkey,
}

#[event]
pub struct EmergencyPause {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct ComplianceViolation {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub violation_type: String,
    pub severity: u8,
    pub timestamp: i64,
}

#[event]
pub struct RiskThresholdBreached {
    pub vault: Pubkey,
    pub metric: String,
    pub current_value: u64,
    pub threshold: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeeCollected {
    pub vault: Pubkey,
    pub fee_type: String,
    pub amount: u64,
    pub collector: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct TrancheRebalanced {
    pub vault: Pubkey,
    pub tranche_index: u8,
    pub old_allocation: u64,
    pub new_allocation: u64,
    pub timestamp: i64,
}

#[event]
pub struct OracleDataReceived {
    pub vault: Pubkey,
    pub oracle: Pubkey,
    pub data_type: String,
    pub value: u64,
    pub confidence: u8,
    pub timestamp: i64,
}

#[event]
pub struct CrossChainMessage {
    pub vault: Pubkey,
    pub destination_chain: String,
    pub message_type: String,
    pub message_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct ZKProofVerified {
    pub vault: Pubkey,
    pub proof_type: String,
    pub verifier: Pubkey,
    pub proof_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct LiquidityProvided {
    pub vault: Pubkey,
    pub provider: Pubkey,
    pub amount: u64,
    pub lp_tokens_minted: u64,
    pub timestamp: i64,
}

#[event]
pub struct LiquidityRemoved {
    pub vault: Pubkey,
    pub provider: Pubkey,
    pub lp_tokens_burned: u64,
    pub assets_returned: u64,
    pub timestamp: i64,
}

#[event]
pub struct YieldDistributed {
    pub vault: Pubkey,
    pub tranche_index: u8,
    pub total_yield: u64,
    pub yield_per_share: u64,
    pub distribution_timestamp: i64,
}

#[event]
pub struct GovernanceProposal {
    pub vault: Pubkey,
    pub proposer: Pubkey,
    pub proposal_id: u64,
    pub proposal_type: String,
    pub voting_deadline: i64,
}

#[event]
pub struct GovernanceVote {
    pub vault: Pubkey,
    pub voter: Pubkey,
    pub proposal_id: u64,
    pub vote: bool,
    pub voting_power: u64,
    pub timestamp: i64,
}

/// PRD: Event for redemption reveal phase
#[event]
pub struct RedemptionRevealed {
    pub vault: Pubkey,
    pub user: Pubkey,
    pub actual_shares_amount: u64,
    pub timestamp: i64,
}

/// PRD: Event for cross-chain anchoring
#[event]
pub struct CrossChainAnchor {
    pub vault: Pubkey,
    pub starknet_proof: [u8; 32],
    pub bitcoin_anchor: [u8; 32],
    pub ethereum_root: [u8; 32],
    pub timestamp: i64,
}

// Constants
pub const MAX_TRANCHES: usize = 5;
pub const MAX_REDEMPTION_QUEUE_SIZE: u32 = 10000;
pub const MIN_NAV_UPDATE_INTERVAL: i64 = 300; // 5 minutes
pub const MAX_NAV_DRIFT_BASIS_POINTS: u64 = 1000; // 10%
pub const DEFAULT_LOCK_PERIOD: u32 = 86400; // 24 hours
pub const MEV_PROTECTION_SLOTS: u64 = 32; // ~13 seconds on Solana

// Validation helpers
impl RedemptionRequest {
    pub fn is_ready_for_processing(&self, current_slot: u64) -> bool {
        current_slot >= self.processing_slot
    }
    
    pub fn is_expired(&self, current_timestamp: i64, expiry_window: i64) -> bool {
        current_timestamp > self.request_timestamp + expiry_window
    }
}

impl NAVData {
    pub fn is_fresh(&self, current_timestamp: i64, max_age: i64) -> bool {
        current_timestamp - self.timestamp <= max_age
    }
    
    pub fn validate_signatures(&self) -> bool {
        // Validate both traditional and post-quantum signatures
        !self.oracle_signature.iter().all(|&x| x == 0) &&
        !self.post_quantum_signature.iter().all(|&x| x == 0)
    }
}

impl ComplianceData {
    pub fn is_valid(&self, current_timestamp: i64, max_age: i64) -> bool {
        self.kyc_verified && 
        (current_timestamp - self.last_verification) <= max_age
    }
    
    pub fn meets_investment_threshold(&self, amount: u64, min_amount: u64) -> bool {
        if self.accredited_investor {
            true
        } else {
            amount >= min_amount
        }
    }
}

impl RiskMetrics {
    pub fn is_within_limits(&self, limits: &RiskLimits) -> bool {
        self.var_95 <= limits.max_var_95 &&
        self.volatility <= limits.max_volatility &&
        self.max_drawdown <= limits.max_drawdown
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct RiskLimits {
    pub max_var_95: u64,
    pub max_volatility: u64,
    pub max_drawdown: u64,
    pub min_sharpe_ratio: i64,
}

// Additional utility traits and implementations
pub trait TrancheCalculations {
    fn calculate_yield(&self, period_days: u32) -> Result<u64>;
    fn calculate_risk_adjusted_return(&self, risk_free_rate: u64) -> Result<i64>;
}

impl TrancheCalculations for crate::Tranche {
    fn calculate_yield(&self, period_days: u32) -> Result<u64> {
        // Annualized yield calculation
        let daily_yield = self.yield_rate / 365;
        Ok(daily_yield * period_days as u64)
    }
    
    fn calculate_risk_adjusted_return(&self, risk_free_rate: u64) -> Result<i64> {
        // Simplified risk-adjusted return calculation
        Ok(self.yield_rate as i64 - risk_free_rate as i64)
    }
}

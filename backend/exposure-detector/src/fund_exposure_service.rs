use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use sha2::{Sha256, Digest};
use petgraph::{Graph, Directed, graph::NodeIndex};
use petgraph::algo::{is_cyclic_directed, toposort};

/// Fund Exposure Detection and Isolation Service
/// PRD Section 4.1: Fund Exposure & Isolation
/// PRD: "Fund-Origin Proof with comprehensive ancestry tracking"
/// PRD: "Recursive zkNAV Flattening for exposure graph detection"
/// PRD: "Cross-fund Ring Detector preventing circular dependencies"
pub struct FundExposureService {
    exposure_graph: RwLock<ExposureGraph>,
    fund_registry: RwLock<HashMap<String, FundMetadata>>,
    circular_dependency_cache: RwLock<HashMap<String, bool>>,
    max_exposure_depth: usize,
    max_circular_exposure: f64, // Percentage
    monitoring_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureGraph {
    pub nodes: HashMap<String, FundNode>,
    pub edges: HashMap<String, Vec<ExposureEdge>>,
    pub total_funds: usize,
    pub total_exposures: usize,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundNode {
    pub fund_id: String,
    pub fund_origin_hash: String,
    pub total_assets: u64,
    pub nav_per_share: u64,
    pub fund_type: FundType,
    pub jurisdiction: String,
    pub creation_timestamp: i64,
    pub last_nav_update: i64,
    pub status: FundStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FundType {
    Primary,      // Original fund with no dependencies
    Derivative,   // Fund that invests in other funds
    Composite,    // Fund with mixed assets and fund exposures
    Synthetic,    // Fund created through derivatives/swaps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FundStatus {
    Active,
    Suspended,
    Liquidating,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureEdge {
    pub from_fund: String,
    pub to_fund: String,
    pub exposure_amount: u64,
    pub exposure_percentage: f64, // Percentage of from_fund's assets
    pub exposure_type: ExposureType,
    pub timestamp: i64,
    pub proof_hash: String, // zkProof of exposure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExposureType {
    DirectInvestment,    // Direct investment in another fund
    DerivativeExposure,  // Exposure through derivatives
    CollateralBacking,   // Fund used as collateral
    SyntheticExposure,   // Synthetic exposure through swaps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundMetadata {
    pub fund_id: String,
    pub name: String,
    pub manager: String,
    pub inception_date: i64,
    pub fund_origin_proof: FundOriginProof,
    pub legal_structure: String,
    pub domicile: String,
    pub base_currency: String,
    pub investment_strategy: String,
    pub target_assets: Vec<String>,
}

/// PRD Section 7: "Fund-Origin Proof: vault_origin_hash = signed snapshot of legal, DAO, and circuit ancestry"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundOriginProof {
    pub vault_origin_hash: String,
    pub legal_ancestry: LegalAncestry,
    pub dao_ancestry: DaoAncestry,
    pub circuit_ancestry: CircuitAncestry,
    pub signed_snapshot: SignedSnapshot,
    pub whitelist_status: WhitelistStatus,
    pub fork_derivation_proof: Option<ForkDerivationProof>,
    pub created_at: i64,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalAncestry {
    pub legal_entity_id: String,
    pub incorporation_documents: Vec<String>,
    pub regulatory_approvals: Vec<String>,
    pub compliance_certifications: Vec<String>,
    pub legal_opinion_hash: String,
    pub jurisdiction_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoAncestry {
    pub governance_contract_address: String,
    pub dao_proposal_history: Vec<String>,
    pub voting_power_distribution: HashMap<String, f64>,
    pub governance_token_address: String,
    pub multisig_signers: Vec<String>,
    pub governance_parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitAncestry {
    pub zknav_circuit_hash: String,
    pub verification_key_hash: String,
    pub circuit_parameters: HashMap<String, serde_json::Value>,
    pub trusted_setup_ceremony: String,
    pub circuit_audit_reports: Vec<String>,
    pub upgrade_history: Vec<CircuitUpgrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitUpgrade {
    pub upgrade_id: String,
    pub old_circuit_hash: String,
    pub new_circuit_hash: String,
    pub upgrade_reason: String,
    pub dao_approval_tx: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedSnapshot {
    pub snapshot_hash: String,
    pub snapshot_data: String,
    pub signatures: Vec<OriginSignature>,
    pub merkle_root: String,
    pub block_height: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginSignature {
    pub signer_address: String,
    pub signer_role: SignerRole,
    pub signature: String,
    pub signature_type: SignatureType,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignerRole {
    LegalEntity,
    DaoMultisig,
    CircuitAuditor,
    RegulatoryApprover,
    TechnicalValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    ECDSA,
    EdDSA,
    Dilithium512, // Post-quantum
    BLS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhitelistStatus {
    ExplicitlyWhitelisted,
    DerivedFromWhitelisted,
    PendingApproval,
    Rejected,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkDerivationProof {
    pub parent_fund_id: String,
    pub fork_reason: String,
    pub derivation_proof: String,
    pub dao_approval_tx: String,
    pub legal_continuity_proof: String,
    pub asset_migration_proof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependencyResult {
    pub circular_detected: bool,
    pub cycle_path: Vec<String>,
    pub total_exposure_in_cycle: u64,
    pub max_exposure_percentage: f64,
    pub risk_level: RiskLevel,
    pub recommended_action: String,
}

/// PRD: "Recursive zkNAV Flattening: nested fund exposure graphs"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveExposureFlattening {
    pub root_fund_id: String,
    pub flattened_exposures: HashMap<String, FlattenedExposure>,
    pub nested_exposure_graphs: Vec<NestedExposureGraph>,
    pub exposure_loops: Vec<ExposureLoop>,
    pub multi_fund_shareholdings: Vec<MultiFundShareholding>,
    pub concentration_analysis: ConcentrationAnalysis,
    pub systemic_risks: Vec<SystemicRisk>,
    pub total_recursion_depth: u32,
    pub weight_threshold_bps: u16,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlattenedExposure {
    pub from_fund: String,
    pub to_fund: String,
    pub direct_weight: f64,
    pub cumulative_weight: f64,
    pub recursion_depth: u32,
    pub exposure_type: ExposureType,
    pub risk_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedExposureGraph {
    pub fund_id: String,
    pub depth: u32,
    pub direct_exposures: Vec<DirectExposureInfo>,
    pub total_exposure_weight: f64,
    pub exposure_concentration: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectExposureInfo {
    pub target_fund: String,
    pub weight_percentage: f64,
    pub exposure_type: ExposureType,
    pub risk_metrics: ExposureRiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureRiskMetrics {
    pub liquidity_risk: f64,
    pub concentration_risk: f64,
    pub counterparty_risk: f64,
    pub market_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureLoop {
    pub loop_id: String,
    pub loop_path: Vec<String>,
    pub total_weight: f64,
    pub loop_depth: u32,
    pub risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiFundShareholding {
    pub fund_id: String,
    pub shareholding_funds: Vec<String>,
    pub total_indirect_weight: f64,
    pub concentration_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcentrationAnalysis {
    pub herfindahl_index: f64,
    pub top_5_concentrations: Vec<(String, f64)>,
    pub total_exposure_weight: f64,
    pub concentration_risk_level: ConcentrationRiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcentrationRiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicRisk {
    pub risk_type: SystemicRiskType,
    pub description: String,
    pub severity: RiskSeverity,
    pub affected_funds: Vec<String>,
    pub mitigation_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemicRiskType {
    HighConcentration,
    ComplexExposureLoop,
    LiquidityRisk,
    CounterpartyRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Advanced Herfindahl-Hirschman Index Analysis Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerfindahlIndexResult {
    pub hhi_score: f64,
    pub market_concentration: MarketConcentration,
    pub dominant_funds: Vec<DominantFund>,
    pub concentration_ratio_cr4: f64,
    pub concentration_ratio_cr8: f64,
    pub entropy_index: f64,
    pub gini_coefficient: f64,
    pub analysis_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketConcentration {
    NoConcentration,
    Unconcentrated,
    ModeratelyConcentrated,
    HighlyConcentrated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominantFund {
    pub fund_id: String,
    pub market_share: f64,
    pub exposure_amount: f64,
    pub dominance_score: f64,
}

/// Advanced Fund-Origin Proof Structures
/// PRD: "fund fork must derive", "explicitly whitelisted", "legal ancestry", "DAO ancestry"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkDerivationResult {
    pub child_fund_id: String,
    pub parent_fund_id: String,
    pub derivation_valid: bool,
    pub child_origin_hash: String,
    pub legal_continuity_verified: bool,
    pub dao_approval_verified: bool,
    pub asset_migration_verified: bool,
    pub derivation_timestamp: i64,
    pub compliance_status: ComplianceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    UnderReview,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhitelistAction {
    Add,
    Remove,
    UpdateConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistAuthorization {
    pub authorizer_address: String,
    pub signature: String,
    pub expiry_timestamp: i64,
    pub conditions: Vec<String>,
    pub authorization_level: AuthorizationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationLevel {
    Admin,
    Governance,
    Legal,
    Regulatory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub fund_id: String,
    pub whitelist_status: WhitelistStatus,
    pub authorized_by: String,
    pub authorization_proof: String,
    pub whitelist_timestamp: i64,
    pub expiry_timestamp: i64,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistResult {
    pub fund_id: String,
    pub action: WhitelistAction,
    pub success: bool,
    pub new_status: WhitelistStatus,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalAncestryVerification {
    pub fund_id: String,
    pub incorporation_verified: bool,
    pub regulatory_approvals_verified: bool,
    pub compliance_certifications_verified: bool,
    pub legal_opinion_verified: bool,
    pub jurisdiction_chain_verified: bool,
    pub overall_legal_validity: bool,
    pub verification_timestamp: i64,
    pub legal_risk_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoAncestryVerification {
    pub fund_id: String,
    pub governance_contract_verified: bool,
    pub proposal_history_verified: bool,
    pub voting_power_verified: bool,
    pub governance_token_verified: bool,
    pub multisig_verified: bool,
    pub parameters_verified: bool,
    pub overall_dao_validity: bool,
    pub verification_timestamp: i64,
    pub dao_decentralization_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,      // <10% circular exposure
    Medium,   // 10-25% circular exposure
    High,     // 25-50% circular exposure
    Critical, // >50% circular exposure
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureAnalysisResult {
    pub fund_id: String,
    pub direct_exposures: Vec<ExposureEdge>,
    pub indirect_exposures: Vec<Vec<ExposureEdge>>, // Paths of indirect exposures
    pub total_exposure_amount: u64,
    pub total_exposure_percentage: f64,
    pub max_depth_reached: usize,
    pub circular_dependencies: Vec<CircularDependencyResult>,
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub overall_risk_level: RiskLevel,
    pub concentration_risk: f64,
    pub liquidity_risk: f64,
    pub counterparty_risk: f64,
    pub systemic_risk: f64,
    pub recommendations: Vec<String>,
}

impl FundExposureService {
    /// Initialize Fund Exposure Service
    pub async fn new(
        max_exposure_depth: usize,
        max_circular_exposure: f64,
    ) -> Result<Self> {
        info!("🕸️ Initializing Fund Exposure Detection Service");
        
        let service = Self {
            exposure_graph: RwLock::new(ExposureGraph {
                nodes: HashMap::new(),
                edges: HashMap::new(),
                total_funds: 0,
                total_exposures: 0,
                last_updated: chrono::Utc::now().timestamp(),
            }),
            fund_registry: RwLock::new(HashMap::new()),
            circular_dependency_cache: RwLock::new(HashMap::new()),
            max_exposure_depth,
            max_circular_exposure,
            monitoring_enabled: true,
        };

        info!("✅ Fund Exposure Service initialized with max depth: {}, max circular: {}%", 
              max_exposure_depth, max_circular_exposure * 100.0);
        Ok(service)
    }

    /// PRD: Register new fund with origin proof
    /// PRD: "Fund-Origin Proof with comprehensive ancestry tracking"
    pub async fn register_fund(
        &self,
        fund_metadata: FundMetadata,
        fund_origin_proof: String,
    ) -> Result<()> {
        info!("📝 Registering fund: {}", fund_metadata.fund_id);

        // Verify fund origin proof
        self.verify_fund_origin_proof(&fund_origin_proof).await?;

        // Create fund node
        let fund_node = FundNode {
            fund_id: fund_metadata.fund_id.clone(),
            fund_origin_hash: fund_origin_proof.clone(),
            total_assets: 0,
            nav_per_share: 1_000_000, // 1.0 with 6 decimals
            fund_type: FundType::Primary, // Will be updated based on exposures
            jurisdiction: fund_metadata.domicile.clone(),
            creation_timestamp: chrono::Utc::now().timestamp(),
            last_nav_update: chrono::Utc::now().timestamp(),
            status: FundStatus::Active,
        };

        // Add to graph and registry
        {
            let mut graph = self.exposure_graph.write().await;
            graph.nodes.insert(fund_metadata.fund_id.clone(), fund_node);
            graph.edges.insert(fund_metadata.fund_id.clone(), Vec::new());
            graph.total_funds += 1;
            graph.last_updated = chrono::Utc::now().timestamp();
        }

        {
            let mut registry = self.fund_registry.write().await;
            registry.insert(fund_metadata.fund_id.clone(), fund_metadata);
        }

        info!("✅ Fund registered successfully: {}", fund_metadata.fund_id);
        Ok(())
    }

    /// PRD: Add exposure between funds
    /// PRD: "Recursive zkNAV Flattening for exposure graph detection"
    pub async fn add_fund_exposure(
        &self,
        from_fund: String,
        to_fund: String,
        exposure_amount: u64,
        exposure_type: ExposureType,
        zk_proof: String,
    ) -> Result<()> {
        info!("🔗 Adding exposure: {} -> {} ({})", from_fund, to_fund, exposure_amount);

        // Verify zkProof of exposure
        self.verify_exposure_proof(&zk_proof, &from_fund, &to_fund, exposure_amount).await?;

        // Calculate exposure percentage
        let from_fund_assets = {
            let graph = self.exposure_graph.read().await;
            graph.nodes.get(&from_fund)
                .map(|node| node.total_assets)
                .unwrap_or(0)
        };

        let exposure_percentage = if from_fund_assets > 0 {
            (exposure_amount as f64 / from_fund_assets as f64) * 100.0
        } else {
            0.0
        };

        // Create exposure edge
        let exposure_edge = ExposureEdge {
            from_fund: from_fund.clone(),
            to_fund: to_fund.clone(),
            exposure_amount,
            exposure_percentage,
            exposure_type,
            timestamp: chrono::Utc::now().timestamp(),
            proof_hash: zk_proof,
        };

        // Add to graph
        {
            let mut graph = self.exposure_graph.write().await;
            if let Some(edges) = graph.edges.get_mut(&from_fund) {
                edges.push(exposure_edge);
                graph.total_exposures += 1;
                graph.last_updated = chrono::Utc::now().timestamp();
            }
        }

        // PRD: Check for circular dependencies
        let circular_result = self.detect_circular_dependency(&from_fund).await?;
        if circular_result.circular_detected {
            warn!("🚨 Circular dependency detected: {:?}", circular_result.cycle_path);
            
            if circular_result.max_exposure_percentage > self.max_circular_exposure {
                error!("❌ Circular exposure exceeds limit: {}%", circular_result.max_exposure_percentage);
                return Err(anyhow::anyhow!("Circular exposure limit exceeded"));
            }
        }

        // Update fund types based on exposures
        self.update_fund_types().await?;

        info!("✅ Exposure added successfully with {}% allocation", exposure_percentage);
        Ok(())
    }

    /// PRD: Detect circular dependencies
    /// PRD: "Cross-fund Ring Detector preventing circular dependencies"
    pub async fn detect_circular_dependency(
        &self,
        fund_id: &str,
    ) -> Result<CircularDependencyResult> {
        info!("🔍 Detecting circular dependencies for fund: {}", fund_id);

        // Check cache first
        {
            let cache = self.circular_dependency_cache.read().await;
            if let Some(&cached_result) = cache.get(fund_id) {
                if !cached_result {
                    return Ok(CircularDependencyResult {
                        circular_detected: false,
                        cycle_path: Vec::new(),
                        total_exposure_in_cycle: 0,
                        max_exposure_percentage: 0.0,
                        risk_level: RiskLevel::Low,
                        recommended_action: "No action required".to_string(),
                    });
                }
            }
        }

        let graph = self.exposure_graph.read().await;
        
        // Use DFS to detect cycles
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        if self.dfs_cycle_detection(&graph, fund_id, &mut visited, &mut rec_stack, &mut path) {
            // Calculate cycle metrics
            let cycle_start_index = path.iter().position(|f| f == fund_id).unwrap_or(0);
            let cycle_path = path[cycle_start_index..].to_vec();
            
            let (total_exposure, max_percentage) = self.calculate_cycle_exposure(&graph, &cycle_path);
            
            let risk_level = match max_percentage {
                p if p > 50.0 => RiskLevel::Critical,
                p if p > 25.0 => RiskLevel::High,
                p if p > 10.0 => RiskLevel::Medium,
                _ => RiskLevel::Low,
            };

            let recommended_action = match risk_level {
                RiskLevel::Critical => "Immediate reduction of circular exposure required".to_string(),
                RiskLevel::High => "Reduce circular exposure within 30 days".to_string(),
                RiskLevel::Medium => "Monitor and consider reducing exposure".to_string(),
                RiskLevel::Low => "Continue monitoring".to_string(),
            };

            // Cache result
            {
                let mut cache = self.circular_dependency_cache.write().await;
                cache.insert(fund_id.to_string(), true);
            }

            Ok(CircularDependencyResult {
                circular_detected: true,
                cycle_path,
                total_exposure_in_cycle: total_exposure,
                max_exposure_percentage: max_percentage,
                risk_level,
                recommended_action,
            })
        } else {
            // Cache negative result
            {
                let mut cache = self.circular_dependency_cache.write().await;
                cache.insert(fund_id.to_string(), false);
            }

            Ok(CircularDependencyResult {
                circular_detected: false,
                cycle_path: Vec::new(),
                total_exposure_in_cycle: 0,
                max_exposure_percentage: 0.0,
                risk_level: RiskLevel::Low,
                recommended_action: "No action required".to_string(),
            })
        }
    }

    /// PRD: Comprehensive exposure analysis
    /// PRD: "Recursive zkNAV Flattening for exposure graph detection"
    pub async fn analyze_fund_exposure(
        &self,
        fund_id: &str,
    ) -> Result<ExposureAnalysisResult> {
        info!("📊 Analyzing exposure for fund: {}", fund_id);

        let graph = self.exposure_graph.read().await;
        
        // Get direct exposures
        let direct_exposures = graph.edges.get(fund_id).cloned().unwrap_or_default();
        
        // Get indirect exposures (recursive)
        let indirect_exposures = self.find_indirect_exposures(&graph, fund_id, self.max_exposure_depth);
        
        // Calculate totals
        let total_direct_exposure: u64 = direct_exposures.iter().map(|e| e.exposure_amount).sum();
        let total_indirect_exposure: u64 = indirect_exposures.iter()
            .flat_map(|path| path.iter())
            .map(|e| e.exposure_amount)
            .sum();
        
        let total_exposure_amount = total_direct_exposure + total_indirect_exposure;
        
        let fund_assets = graph.nodes.get(fund_id).map(|n| n.total_assets).unwrap_or(0);
        let total_exposure_percentage = if fund_assets > 0 {
            (total_exposure_amount as f64 / fund_assets as f64) * 100.0
        } else {
            0.0
        };

        // Detect circular dependencies
        let circular_dependencies = vec![self.detect_circular_dependency(fund_id).await?];
        
        // Risk assessment
        let risk_assessment = self.assess_exposure_risk(
            &direct_exposures,
            &indirect_exposures,
            total_exposure_percentage,
            &circular_dependencies,
        );

        Ok(ExposureAnalysisResult {
            fund_id: fund_id.to_string(),
            direct_exposures,
            indirect_exposures,
            total_exposure_amount,
            total_exposure_percentage,
            max_depth_reached: self.max_exposure_depth,
            circular_dependencies,
            risk_assessment,
        })
    }

    // Private helper methods
    async fn verify_fund_origin_proof(&self, proof: &str) -> Result<()> {
        // TODO: Implement actual zkProof verification of fund origin
        info!("🔍 Verifying fund origin proof");
        Ok(())
    }

    async fn verify_exposure_proof(&self, proof: &str, from_fund: &str, to_fund: &str, amount: u64) -> Result<()> {
        // TODO: Implement actual zkProof verification of exposure
        info!("🔍 Verifying exposure proof for {} -> {}", from_fund, to_fund);
        Ok(())
    }

    fn dfs_cycle_detection(
        &self,
        graph: &ExposureGraph,
        fund_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        visited.insert(fund_id.to_string());
        rec_stack.insert(fund_id.to_string());
        path.push(fund_id.to_string());

        if let Some(edges) = graph.edges.get(fund_id) {
            for edge in edges {
                if !visited.contains(&edge.to_fund) {
                    if self.dfs_cycle_detection(graph, &edge.to_fund, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(&edge.to_fund) {
                    path.push(edge.to_fund.clone());
                    return true;
                }
            }
        }

        rec_stack.remove(fund_id);
        path.pop();
        false
    }

    fn calculate_cycle_exposure(&self, graph: &ExposureGraph, cycle_path: &[String]) -> (u64, f64) {
        let mut total_exposure = 0u64;
        let mut max_percentage = 0.0f64;

        for i in 0..cycle_path.len() {
            let from_fund = &cycle_path[i];
            let to_fund = &cycle_path[(i + 1) % cycle_path.len()];

            if let Some(edges) = graph.edges.get(from_fund) {
                for edge in edges {
                    if edge.to_fund == *to_fund {
                        total_exposure += edge.exposure_amount;
                        max_percentage = max_percentage.max(edge.exposure_percentage);
                    }
                }
            }
        }

        (total_exposure, max_percentage)
    }

    fn find_indirect_exposures(
        &self,
        graph: &ExposureGraph,
        fund_id: &str,
        max_depth: usize,
    ) -> Vec<Vec<ExposureEdge>> {
        let mut indirect_exposures = Vec::new();
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        // Start BFS from direct exposures
        if let Some(direct_edges) = graph.edges.get(fund_id) {
            for edge in direct_edges {
                queue.push_back((vec![edge.clone()], 1));
            }
        }

        while let Some((path, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let last_edge = path.last().unwrap();
            let current_fund = &last_edge.to_fund;

            if visited.contains(current_fund) {
                continue;
            }
            visited.insert(current_fund.clone());

            if let Some(edges) = graph.edges.get(current_fund) {
                for edge in edges {
                    let mut new_path = path.clone();
                    new_path.push(edge.clone());
                    
                    if depth + 1 < max_depth {
                        queue.push_back((new_path.clone(), depth + 1));
                    }
                    
                    indirect_exposures.push(new_path);
                }
            }
        }

        indirect_exposures
    }

    fn assess_exposure_risk(
        &self,
        direct_exposures: &[ExposureEdge],
        indirect_exposures: &[Vec<ExposureEdge>],
        total_exposure_percentage: f64,
        circular_dependencies: &[CircularDependencyResult],
    ) -> RiskAssessment {
        // Calculate concentration risk
        let concentration_risk = if direct_exposures.len() <= 3 {
            total_exposure_percentage / 100.0
        } else {
            total_exposure_percentage / 200.0 // Diversification benefit
        };

        // Calculate liquidity risk (simplified)
        let liquidity_risk = total_exposure_percentage / 100.0 * 0.5;

        // Calculate counterparty risk
        let unique_counterparties = direct_exposures.iter()
            .map(|e| &e.to_fund)
            .collect::<HashSet<_>>()
            .len();
        let counterparty_risk = if unique_counterparties <= 5 {
            0.8
        } else {
            0.3
        };

        // Calculate systemic risk
        let systemic_risk = circular_dependencies.iter()
            .map(|cd| cd.max_exposure_percentage / 100.0)
            .fold(0.0, f64::max);

        // Overall risk level
        let overall_risk_level = match (concentration_risk + liquidity_risk + counterparty_risk + systemic_risk) / 4.0 {
            r if r > 0.5 => RiskLevel::Critical,
            r if r > 0.3 => RiskLevel::High,
            r if r > 0.15 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        };

        // Generate recommendations
        let mut recommendations = Vec::new();
        if concentration_risk > 0.3 {
            recommendations.push("Consider diversifying exposures across more funds".to_string());
        }
        if counterparty_risk > 0.5 {
            recommendations.push("Reduce concentration in single counterparties".to_string());
        }
        if systemic_risk > 0.2 {
            recommendations.push("Address circular dependencies to reduce systemic risk".to_string());
        }

        RiskAssessment {
            overall_risk_level,
            concentration_risk,
            liquidity_risk,
            counterparty_risk,
            systemic_risk,
            recommendations,
        }
    }

    async fn update_fund_types(&self) -> Result<()> {
        // Update fund types based on their exposure patterns
        let mut graph = self.exposure_graph.write().await;

        for (fund_id, node) in graph.nodes.iter_mut() {
            let has_exposures = graph.edges.get(fund_id)
                .map(|edges| !edges.is_empty())
                .unwrap_or(false);

            node.fund_type = if has_exposures {
                FundType::Derivative
            } else {
                FundType::Primary
            };
        }

        Ok(())
    }

    /// PRD: "Recursive zkNAV Flattening: nested fund exposure graphs"
    /// PRD: "weight > X%, exposure loops, multi-fund shareholding"
    pub async fn recursive_zknav_flattening(
        &self,
        root_fund_id: String,
        weight_threshold_bps: u16, // basis points (e.g., 1000 = 10%)
        max_recursion_depth: u32,
    ) -> Result<RecursiveExposureFlattening> {
        info!("🔄 Starting recursive zkNAV flattening for fund: {}", root_fund_id);

        let mut flattened_exposures = HashMap::new();
        let mut nested_exposure_graphs = Vec::new();
        let mut exposure_loops = Vec::new();
        let mut multi_fund_shareholdings = Vec::new();
        let mut visited_funds = HashSet::new();
        let mut recursion_stack = Vec::new();

        // Start recursive flattening
        self.flatten_fund_exposures_recursive(
            &root_fund_id,
            1.0, // 100% weight for root fund
            0,
            max_recursion_depth,
            weight_threshold_bps,
            &mut flattened_exposures,
            &mut nested_exposure_graphs,
            &mut exposure_loops,
            &mut multi_fund_shareholdings,
            &mut visited_funds,
            &mut recursion_stack,
        ).await?;

        // Analyze exposure concentration
        let concentration_analysis = self.analyze_exposure_concentration(&flattened_exposures).await?;

        // Detect systemic risks
        let systemic_risks = self.detect_systemic_risks(&flattened_exposures, &exposure_loops).await?;

        let result = RecursiveExposureFlattening {
            root_fund_id,
            flattened_exposures,
            nested_exposure_graphs,
            exposure_loops,
            multi_fund_shareholdings,
            concentration_analysis,
            systemic_risks,
            total_recursion_depth: recursion_stack.len() as u32,
            weight_threshold_bps,
            timestamp: chrono::Utc::now().timestamp(),
        };

        info!("✅ Recursive zkNAV flattening completed - Found {} nested exposures, {} loops",
              result.flattened_exposures.len(), result.exposure_loops.len());

        Ok(result)
    }

    /// Recursive function to flatten fund exposures
    async fn flatten_fund_exposures_recursive(
        &self,
        fund_id: &str,
        cumulative_weight: f64,
        current_depth: u32,
        max_depth: u32,
        weight_threshold_bps: u16,
        flattened_exposures: &mut HashMap<String, FlattenedExposure>,
        nested_graphs: &mut Vec<NestedExposureGraph>,
        exposure_loops: &mut Vec<ExposureLoop>,
        multi_fund_shareholdings: &mut Vec<MultiFundShareholding>,
        visited_funds: &mut HashSet<String>,
        recursion_stack: &mut Vec<String>,
    ) -> Result<()> {
        // Check recursion limits
        if current_depth >= max_depth {
            return Ok(());
        }

        // Check for exposure loops
        if recursion_stack.contains(&fund_id.to_string()) {
            let loop_start_index = recursion_stack.iter().position(|f| f == fund_id).unwrap();
            let mut loop_path = recursion_stack[loop_start_index..].to_vec();
            loop_path.push(fund_id.to_string());

            exposure_loops.push(ExposureLoop {
                loop_id: format!("loop_{}_{}", fund_id, chrono::Utc::now().timestamp()),
                loop_path,
                total_weight: cumulative_weight,
                loop_depth: current_depth,
                risk_score: self.calculate_loop_risk_score(cumulative_weight, current_depth),
            });
            return Ok(());
        }

        recursion_stack.push(fund_id.to_string());

        // Get fund exposures
        let graph = self.exposure_graph.read().await;
        if let Some(fund_exposures) = graph.edges.get(fund_id) {
            let mut current_graph = NestedExposureGraph {
                fund_id: fund_id.to_string(),
                depth: current_depth,
                direct_exposures: Vec::new(),
                total_exposure_weight: 0.0,
                exposure_concentration: 0.0,
            };

            for exposure in fund_exposures {
                let exposure_weight_bps = (exposure.exposure_percentage * 100.0) as u16;

                // Only process exposures above threshold
                if exposure_weight_bps >= weight_threshold_bps {
                    let nested_weight = cumulative_weight * (exposure.exposure_percentage / 100.0);

                    // Add to flattened exposures
                    let flattened_key = format!("{}_{}", fund_id, exposure.to_fund);
                    flattened_exposures.insert(flattened_key.clone(), FlattenedExposure {
                        from_fund: fund_id.to_string(),
                        to_fund: exposure.to_fund.clone(),
                        direct_weight: exposure.exposure_percentage / 100.0,
                        cumulative_weight: nested_weight,
                        recursion_depth: current_depth,
                        exposure_type: exposure.exposure_type.clone(),
                        risk_contribution: self.calculate_risk_contribution(nested_weight, current_depth),
                    });

                    // Add to current graph
                    current_graph.direct_exposures.push(DirectExposureInfo {
                        target_fund: exposure.to_fund.clone(),
                        weight_percentage: exposure.exposure_percentage,
                        exposure_type: exposure.exposure_type.clone(),
                        risk_metrics: self.calculate_exposure_risk_metrics(exposure).await?,
                    });
                    current_graph.total_exposure_weight += exposure.exposure_percentage / 100.0;

                    // Check for multi-fund shareholding
                    if visited_funds.contains(&exposure.to_fund) {
                        multi_fund_shareholdings.push(MultiFundShareholding {
                            fund_id: exposure.to_fund.clone(),
                            shareholding_funds: recursion_stack.clone(),
                            total_indirect_weight: nested_weight,
                            concentration_risk: self.calculate_concentration_risk(nested_weight),
                        });
                    }

                    visited_funds.insert(exposure.to_fund.clone());

                    // Recurse into nested fund
                    self.flatten_fund_exposures_recursive(
                        &exposure.to_fund,
                        nested_weight,
                        current_depth + 1,
                        max_depth,
                        weight_threshold_bps,
                        flattened_exposures,
                        nested_graphs,
                        exposure_loops,
                        multi_fund_shareholdings,
                        visited_funds,
                        recursion_stack,
                    ).await?;
                }
            }

            // Calculate exposure concentration for this level
            current_graph.exposure_concentration = self.calculate_exposure_concentration(&current_graph.direct_exposures);
            nested_graphs.push(current_graph);
        }

        recursion_stack.pop();
        Ok(())
    }

    // Helper methods for recursive flattening
    fn calculate_loop_risk_score(&self, weight: f64, depth: u32) -> f64 {
        weight * (depth as f64).log2() * 10.0
    }

    fn calculate_risk_contribution(&self, weight: f64, depth: u32) -> f64 {
        weight * (1.0 + (depth as f64) * 0.1)
    }

    async fn calculate_exposure_risk_metrics(&self, exposure: &ExposureEdge) -> Result<ExposureRiskMetrics> {
        Ok(ExposureRiskMetrics {
            liquidity_risk: 0.1,
            concentration_risk: exposure.exposure_percentage / 100.0,
            counterparty_risk: 0.05,
            market_risk: 0.08,
        })
    }

    fn calculate_concentration_risk(&self, weight: f64) -> f64 {
        if weight > 0.5 { 1.0 } else if weight > 0.25 { 0.7 } else if weight > 0.1 { 0.4 } else { 0.1 }
    }

    fn calculate_exposure_concentration(&self, exposures: &[DirectExposureInfo]) -> f64 {
        if exposures.is_empty() { return 0.0; }

        let total_weight: f64 = exposures.iter().map(|e| e.weight_percentage / 100.0).sum();
        let hhi: f64 = exposures.iter()
            .map(|e| (e.weight_percentage / 100.0 / total_weight).powi(2))
            .sum();

        hhi
    }

    async fn analyze_exposure_concentration(&self, flattened_exposures: &HashMap<String, FlattenedExposure>) -> Result<ConcentrationAnalysis> {
        let mut fund_concentrations = HashMap::new();
        let mut total_exposure = 0.0;

        // Calculate concentration by target fund
        for exposure in flattened_exposures.values() {
            *fund_concentrations.entry(exposure.to_fund.clone()).or_insert(0.0) += exposure.cumulative_weight;
            total_exposure += exposure.cumulative_weight;
        }

        // Calculate Herfindahl-Hirschman Index (HHI)
        let hhi = fund_concentrations.values()
            .map(|&weight| (weight * 10000.0).powi(2))
            .sum::<f64>() / 10000.0;

        // Find top concentrations
        let mut sorted_concentrations: Vec<_> = fund_concentrations.into_iter().collect();
        sorted_concentrations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(ConcentrationAnalysis {
            herfindahl_index: hhi,
            top_5_concentrations: sorted_concentrations.into_iter().take(5).collect(),
            total_exposure_weight: total_exposure,
            concentration_risk_level: self.classify_concentration_risk(hhi),
        })
    }

    async fn detect_systemic_risks(&self, flattened_exposures: &HashMap<String, FlattenedExposure>, exposure_loops: &[ExposureLoop]) -> Result<Vec<SystemicRisk>> {
        let mut systemic_risks = Vec::new();

        // Risk 1: High concentration in single fund
        for exposure in flattened_exposures.values() {
            if exposure.cumulative_weight > 0.25 { // 25% threshold
                systemic_risks.push(SystemicRisk {
                    risk_type: SystemicRiskType::HighConcentration,
                    description: format!("High exposure concentration: {:.1}% in fund {}",
                                       exposure.cumulative_weight * 100.0, exposure.to_fund),
                    severity: if exposure.cumulative_weight > 0.5 { RiskSeverity::Critical } else { RiskSeverity::High },
                    affected_funds: vec![exposure.from_fund.clone(), exposure.to_fund.clone()],
                    mitigation_recommendations: vec![
                        "Diversify fund exposures".to_string(),
                        "Implement exposure limits".to_string(),
                        "Monitor concentration metrics".to_string(),
                    ],
                });
            }
        }

        // Risk 2: Complex exposure loops
        for exposure_loop in exposure_loops {
            if exposure_loop.loop_depth > 3 || exposure_loop.total_weight > 0.1 {
                systemic_risks.push(SystemicRisk {
                    risk_type: SystemicRiskType::ComplexExposureLoop,
                    description: format!("Complex exposure loop detected with depth {} and weight {:.1}%",
                                       exposure_loop.loop_depth, exposure_loop.total_weight * 100.0),
                    severity: if exposure_loop.loop_depth > 5 { RiskSeverity::Critical } else { RiskSeverity::Medium },
                    affected_funds: exposure_loop.loop_path.clone(),
                    mitigation_recommendations: vec![
                        "Break circular dependencies".to_string(),
                        "Implement loop detection monitoring".to_string(),
                        "Set maximum recursion depth limits".to_string(),
                    ],
                });
            }
        }

        Ok(systemic_risks)
    }

    /// PRD: "fund fork must derive" - Advanced Fund Fork Derivation System
    pub async fn verify_fund_fork_derivation(
        &self,
        child_fund_id: String,
        parent_fund_id: String,
        derivation_proof: ForkDerivationProof,
    ) -> Result<ForkDerivationResult> {
        info!("🔍 Verifying fund fork derivation: {} -> {}", parent_fund_id, child_fund_id);

        // Get parent fund origin proof
        let parent_metadata = self.get_fund_metadata(&parent_fund_id).await?;
        let parent_origin = &parent_metadata.fund_origin_proof;

        // Verify legal continuity
        let legal_continuity_valid = self.verify_legal_continuity(
            &parent_origin.legal_ancestry,
            &derivation_proof.legal_continuity_proof,
        ).await?;

        // Verify DAO approval for fork
        let dao_approval_valid = self.verify_dao_fork_approval(
            &parent_origin.dao_ancestry,
            &derivation_proof.dao_approval_tx,
        ).await?;

        // Verify asset migration integrity
        let asset_migration_valid = self.verify_asset_migration_proof(
            &derivation_proof.asset_migration_proof,
        ).await?;

        // Generate child fund origin hash
        let child_origin_hash = self.generate_derived_origin_hash(
            &parent_origin.vault_origin_hash,
            &derivation_proof,
        ).await?;

        let result = ForkDerivationResult {
            child_fund_id,
            parent_fund_id,
            derivation_valid: legal_continuity_valid && dao_approval_valid && asset_migration_valid,
            child_origin_hash,
            legal_continuity_verified: legal_continuity_valid,
            dao_approval_verified: dao_approval_valid,
            asset_migration_verified: asset_migration_valid,
            derivation_timestamp: chrono::Utc::now().timestamp(),
            compliance_status: if legal_continuity_valid && dao_approval_valid && asset_migration_valid {
                ComplianceStatus::Compliant
            } else {
                ComplianceStatus::NonCompliant
            },
        };

        info!("✅ Fund fork derivation verification completed - Valid: {}", result.derivation_valid);
        Ok(result)
    }

    /// PRD: "explicitly whitelisted" - Advanced Fund Whitelisting System
    pub async fn manage_fund_whitelist(
        &self,
        fund_id: String,
        whitelist_action: WhitelistAction,
        authorization: WhitelistAuthorization,
    ) -> Result<WhitelistResult> {
        info!("📋 Managing fund whitelist: {} - {:?}", fund_id, whitelist_action);

        // Verify authorization
        let auth_valid = self.verify_whitelist_authorization(&authorization).await?;
        require!(auth_valid, "Invalid whitelist authorization");

        let mut whitelist = self.fund_whitelist.write().await;

        let result = match whitelist_action {
            WhitelistAction::Add => {
                let whitelist_entry = WhitelistEntry {
                    fund_id: fund_id.clone(),
                    whitelist_status: WhitelistStatus::ExplicitlyWhitelisted,
                    authorized_by: authorization.authorizer_address.clone(),
                    authorization_proof: authorization.signature.clone(),
                    whitelist_timestamp: chrono::Utc::now().timestamp(),
                    expiry_timestamp: authorization.expiry_timestamp,
                    conditions: authorization.conditions.clone(),
                };

                whitelist.insert(fund_id.clone(), whitelist_entry);

                WhitelistResult {
                    fund_id,
                    action: whitelist_action,
                    success: true,
                    new_status: WhitelistStatus::ExplicitlyWhitelisted,
                    message: "Fund successfully added to whitelist".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                }
            },
            WhitelistAction::Remove => {
                whitelist.remove(&fund_id);

                WhitelistResult {
                    fund_id,
                    action: whitelist_action,
                    success: true,
                    new_status: WhitelistStatus::Rejected,
                    message: "Fund removed from whitelist".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                }
            },
            WhitelistAction::UpdateConditions => {
                if let Some(entry) = whitelist.get_mut(&fund_id) {
                    entry.conditions = authorization.conditions.clone();
                    entry.authorized_by = authorization.authorizer_address.clone();

                    WhitelistResult {
                        fund_id,
                        action: whitelist_action,
                        success: true,
                        new_status: entry.whitelist_status.clone(),
                        message: "Fund whitelist conditions updated".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                } else {
                    WhitelistResult {
                        fund_id,
                        action: whitelist_action,
                        success: false,
                        new_status: WhitelistStatus::Rejected,
                        message: "Fund not found in whitelist".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                    }
                }
            },
        };

        info!("✅ Fund whitelist management completed - Success: {}", result.success);
        Ok(result)
    }

    /// PRD: "legal ancestry" - Advanced Legal Ancestry Verification
    pub async fn verify_comprehensive_legal_ancestry(
        &self,
        fund_id: String,
        legal_ancestry: LegalAncestry,
    ) -> Result<LegalAncestryVerification> {
        info!("⚖️ Verifying comprehensive legal ancestry for fund: {}", fund_id);

        // Verify incorporation documents
        let incorporation_valid = self.verify_incorporation_documents(
            &legal_ancestry.incorporation_documents,
        ).await?;

        // Verify regulatory approvals
        let regulatory_valid = self.verify_regulatory_approvals(
            &legal_ancestry.regulatory_approvals,
        ).await?;

        // Verify compliance certifications
        let compliance_valid = self.verify_compliance_certifications(
            &legal_ancestry.compliance_certifications,
        ).await?;

        // Verify legal opinion integrity
        let legal_opinion_valid = self.verify_legal_opinion_hash(
            &legal_ancestry.legal_opinion_hash,
        ).await?;

        // Verify jurisdiction chain
        let jurisdiction_valid = self.verify_jurisdiction_chain(
            &legal_ancestry.jurisdiction_chain,
        ).await?;

        let verification = LegalAncestryVerification {
            fund_id,
            incorporation_verified: incorporation_valid,
            regulatory_approvals_verified: regulatory_valid,
            compliance_certifications_verified: compliance_valid,
            legal_opinion_verified: legal_opinion_valid,
            jurisdiction_chain_verified: jurisdiction_valid,
            overall_legal_validity: incorporation_valid && regulatory_valid && compliance_valid && legal_opinion_valid && jurisdiction_valid,
            verification_timestamp: chrono::Utc::now().timestamp(),
            legal_risk_score: self.calculate_legal_risk_score(
                incorporation_valid,
                regulatory_valid,
                compliance_valid,
                legal_opinion_valid,
                jurisdiction_valid,
            ),
        };

        info!("✅ Legal ancestry verification completed - Valid: {}", verification.overall_legal_validity);
        Ok(verification)
    }

    /// PRD: "DAO ancestry" - Advanced DAO Ancestry Verification
    pub async fn verify_comprehensive_dao_ancestry(
        &self,
        fund_id: String,
        dao_ancestry: DaoAncestry,
    ) -> Result<DaoAncestryVerification> {
        info!("🏛️ Verifying comprehensive DAO ancestry for fund: {}", fund_id);

        // Verify governance contract integrity
        let governance_contract_valid = self.verify_governance_contract(
            &dao_ancestry.governance_contract_address,
        ).await?;

        // Verify proposal history integrity
        let proposal_history_valid = self.verify_dao_proposal_history(
            &dao_ancestry.dao_proposal_history,
        ).await?;

        // Verify voting power distribution
        let voting_power_valid = self.verify_voting_power_distribution(
            &dao_ancestry.voting_power_distribution,
        ).await?;

        // Verify governance token authenticity
        let governance_token_valid = self.verify_governance_token(
            &dao_ancestry.governance_token_address,
        ).await?;

        // Verify multisig signers
        let multisig_valid = self.verify_multisig_signers(
            &dao_ancestry.multisig_signers,
        ).await?;

        // Verify governance parameters
        let parameters_valid = self.verify_governance_parameters(
            &dao_ancestry.governance_parameters,
        ).await?;

        let verification = DaoAncestryVerification {
            fund_id,
            governance_contract_verified: governance_contract_valid,
            proposal_history_verified: proposal_history_valid,
            voting_power_verified: voting_power_valid,
            governance_token_verified: governance_token_valid,
            multisig_verified: multisig_valid,
            parameters_verified: parameters_valid,
            overall_dao_validity: governance_contract_valid && proposal_history_valid && voting_power_valid && governance_token_valid && multisig_valid && parameters_valid,
            verification_timestamp: chrono::Utc::now().timestamp(),
            dao_decentralization_score: self.calculate_dao_decentralization_score(&dao_ancestry),
        };

        info!("✅ DAO ancestry verification completed - Valid: {}", verification.overall_dao_validity);
        Ok(verification)
    }

    fn classify_concentration_risk(&self, hhi: f64) -> ConcentrationRiskLevel {
        if hhi > 0.25 { ConcentrationRiskLevel::High }
        else if hhi > 0.15 { ConcentrationRiskLevel::Medium }
        else { ConcentrationRiskLevel::Low }
    }

    /// PRD: Advanced Herfindahl-Hirschman Index calculation for concentration measurement
    /// Standalone function for external use and advanced concentration analysis
    pub async fn calculate_herfindahl_index(
        &self,
        fund_exposures: HashMap<String, f64>,
    ) -> Result<HerfindahlIndexResult> {
        info!("📊 Calculating advanced Herfindahl-Hirschman Index for concentration analysis");

        let total_exposure: f64 = fund_exposures.values().sum();

        if total_exposure == 0.0 {
            return Ok(HerfindahlIndexResult {
                hhi_score: 0.0,
                market_concentration: MarketConcentration::NoConcentration,
                dominant_funds: Vec::new(),
                concentration_ratio_cr4: 0.0,
                concentration_ratio_cr8: 0.0,
                entropy_index: 0.0,
                gini_coefficient: 0.0,
                analysis_timestamp: chrono::Utc::now().timestamp(),
            });
        }

        // Calculate HHI (sum of squared market shares in basis points)
        let hhi_score: f64 = fund_exposures.values()
            .map(|&exposure| {
                let market_share = exposure / total_exposure;
                market_share * market_share * 10000.0 // Convert to basis points
            })
            .sum();

        // Calculate concentration ratios (CR4 and CR8)
        let mut sorted_exposures: Vec<_> = fund_exposures.iter().collect();
        sorted_exposures.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());

        let cr4 = sorted_exposures.iter()
            .take(4)
            .map(|(_, &exposure)| exposure / total_exposure)
            .sum::<f64>() * 100.0;

        let cr8 = sorted_exposures.iter()
            .take(8)
            .map(|(_, &exposure)| exposure / total_exposure)
            .sum::<f64>() * 100.0;

        // Calculate entropy index (measure of diversity)
        let entropy_index = -fund_exposures.values()
            .map(|&exposure| {
                let share = exposure / total_exposure;
                if share > 0.0 {
                    share * share.ln()
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        // Calculate Gini coefficient (measure of inequality)
        let gini_coefficient = self.calculate_gini_coefficient(&fund_exposures).await?;

        // Identify dominant funds (>10% market share)
        let dominant_funds: Vec<DominantFund> = sorted_exposures.iter()
            .filter(|(_, &exposure)| exposure / total_exposure > 0.1)
            .map(|(fund_id, &exposure)| DominantFund {
                fund_id: fund_id.to_string(),
                market_share: exposure / total_exposure,
                exposure_amount: exposure,
                dominance_score: self.calculate_dominance_score(exposure / total_exposure),
            })
            .collect();

        // Classify market concentration
        let market_concentration = self.classify_market_concentration(hhi_score, cr4);

        let result = HerfindahlIndexResult {
            hhi_score,
            market_concentration,
            dominant_funds,
            concentration_ratio_cr4: cr4,
            concentration_ratio_cr8: cr8,
            entropy_index,
            gini_coefficient,
            analysis_timestamp: chrono::Utc::now().timestamp(),
        };

        info!("✅ HHI calculation completed - Score: {:.2}, Concentration: {:?}", hhi_score, market_concentration);
        Ok(result)
    }

    /// Calculate Gini coefficient for inequality measurement
    async fn calculate_gini_coefficient(&self, exposures: &HashMap<String, f64>) -> Result<f64> {
        let mut values: Vec<f64> = exposures.values().cloned().collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;

        if mean == 0.0 {
            return Ok(0.0);
        }

        let mut sum_diff = 0.0;
        for i in 0..values.len() {
            for j in 0..values.len() {
                sum_diff += (values[i] - values[j]).abs();
            }
        }

        let gini = sum_diff / (2.0 * n * n * mean);
        Ok(gini)
    }

    /// Calculate dominance score for individual funds
    fn calculate_dominance_score(&self, market_share: f64) -> f64 {
        // Exponential scoring for dominance (higher scores for larger shares)
        (market_share * 10.0).exp() / 100.0
    }

    /// Classify market concentration based on HHI and CR4
    fn classify_market_concentration(&self, hhi: f64, cr4: f64) -> MarketConcentration {
        match hhi {
            h if h < 1500.0 => MarketConcentration::Unconcentrated,
            h if h < 2500.0 => MarketConcentration::ModeratelyConcentrated,
            _ => MarketConcentration::HighlyConcentrated,
        }
    }
}

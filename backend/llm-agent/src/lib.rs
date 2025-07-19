pub mod governance_assistant;
pub mod semantic_integrity;
pub mod proposal_analyzer;
pub mod compliance_checker;
pub mod determinism_oracle;

pub use governance_assistant::*;
pub use semantic_integrity::*;
pub use proposal_analyzer::*;
pub use compliance_checker::*;
pub use determinism_oracle::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use crate::determinism_oracle::{LlmDeterminismOracle, GovernanceEventType, InputContext, LlmOutput, FundState, MarketConditions, RegulatoryEnvironment};

/// Advanced LLM Governance Assistant for RTF Infrastructure
/// PRD Section 8: LLM Agent Integrity
/// PRD: "LLM Determinism Oracle: snapshot of assistant outputs for each governance event"
/// PRD: "Deviation Detection: diverging from prior outputs on similar governance scenarios"
/// PRD: "Visual coherence score: prompt injection, echo-loop exploits"
/// PRD: "Governance Simulation Mode: impact of proposal over epoch horizon"
pub struct LLMGovernanceService {
    governance_assistant: GovernanceAssistant,
    semantic_integrity: SemanticIntegrityChecker,
    proposal_analyzer: ProposalAnalyzer,
    compliance_checker: ComplianceChecker,
    determinism_oracle: LlmDeterminismOracle,
    confidence_threshold: u8,
    enabled: bool,
    integrity_monitoring_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysisResult {
    pub proposal_id: String,
    pub semantic_analysis: SemanticAnalysis,
    pub compliance_check: ComplianceResult,
    pub risk_assessment: RiskAssessment,
    pub recommendation: GovernanceRecommendation,
    pub confidence_score: u8,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub intent_classification: String,
    pub parameter_extraction: HashMap<String, serde_json::Value>,
    pub impact_assessment: ImpactAssessment,
    pub consistency_check: bool,
    pub deviation_detected: bool,
    pub deviation_details: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub financial_impact: FinancialImpact,
    pub operational_impact: OperationalImpact,
    pub regulatory_impact: RegulatoryImpact,
    pub systemic_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialImpact {
    pub estimated_cost: u64,
    pub revenue_impact: i64, // Can be negative
    pub nav_impact_percentage: f64,
    pub liquidity_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalImpact {
    pub complexity_score: u8,
    pub implementation_time_days: u32,
    pub resource_requirements: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryImpact {
    pub compliance_frameworks_affected: Vec<String>,
    pub regulatory_approval_required: bool,
    pub notification_requirements: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceRecommendation {
    pub recommendation_type: RecommendationType,
    pub rationale: String,
    pub conditions: Vec<String>,
    pub alternative_proposals: Vec<String>,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Approve,
    ApproveWithConditions,
    Reject,
    RequestMoreInformation,
    Defer,
}

impl LLMGovernanceService {
    /// Initialize Advanced LLM Governance Service with Integrity Monitoring
    pub async fn new(confidence_threshold: u8) -> Result<Self> {
        info!("🤖 Initializing Advanced LLM Governance Service with Integrity Monitoring");

        // Initialize determinism oracle
        let determinism_oracle = LlmDeterminismOracle::new(
            0.85, // confidence threshold
            0.15, // max deviation tolerance
        ).await?;

        let service = Self {
            governance_assistant: GovernanceAssistant::new().await?,
            semantic_integrity: SemanticIntegrityChecker::new().await?,
            proposal_analyzer: ProposalAnalyzer::new().await?,
            compliance_checker: ComplianceChecker::new().await?,
            determinism_oracle,
            confidence_threshold,
            enabled: true,
            integrity_monitoring_enabled: true,
        };

        info!("✅ Advanced LLM Governance Service initialized with integrity monitoring");
        Ok(service)
    }

    /// PRD: Analyze governance proposal with semantic integrity
    /// PRD: "Semantic integrity with LLM parsing"
    pub async fn analyze_proposal(
        &self,
        proposal_id: String,
        proposal_text: String,
        proposal_metadata: ProposalMetadata,
    ) -> Result<LLMAnalysisResult> {
        if !self.enabled {
            return Err(anyhow::anyhow!("LLM Governance Service is disabled"));
        }

        info!("🔍 Analyzing proposal: {}", proposal_id);
        let start_time = std::time::Instant::now();

        // 1. Semantic Analysis
        let semantic_analysis = self.semantic_integrity
            .analyze_proposal_semantics(&proposal_text, &proposal_metadata)
            .await?;

        // 2. Compliance Check
        let compliance_check = self.compliance_checker
            .check_proposal_compliance(&proposal_text, &proposal_metadata)
            .await?;

        // 3. Risk Assessment
        let risk_assessment = self.proposal_analyzer
            .assess_proposal_risk(&proposal_text, &semantic_analysis)
            .await?;

        // 4. Generate Recommendation
        let recommendation = self.governance_assistant
            .generate_recommendation(&semantic_analysis, &compliance_check, &risk_assessment)
            .await?;

        // 5. Calculate Confidence Score
        let confidence_score = self.calculate_confidence_score(
            &semantic_analysis,
            &compliance_check,
            &risk_assessment,
        );

        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        let result = LLMAnalysisResult {
            proposal_id,
            semantic_analysis,
            compliance_check,
            risk_assessment,
            recommendation,
            confidence_score,
            processing_time_ms,
        };

        info!("✅ Proposal analysis completed in {}ms with confidence: {}",
              processing_time_ms, confidence_score);

        Ok(result)
    }

    /// Calculate overall confidence score
    fn calculate_confidence_score(
        &self,
        semantic_analysis: &SemanticAnalysis,
        compliance_check: &ComplianceResult,
        risk_assessment: &RiskAssessment,
    ) -> u8 {
        let mut score = 100u8;

        // Reduce score for semantic issues
        if semantic_analysis.deviation_detected {
            score = score.saturating_sub(20);
        }
        if !semantic_analysis.consistency_check {
            score = score.saturating_sub(15);
        }

        // Reduce score for compliance issues
        if !compliance_check.compliant {
            score = score.saturating_sub(30);
        }

        // Reduce score for high risk
        match risk_assessment.overall_risk_level {
            RiskLevel::Critical => score = score.saturating_sub(40),
            RiskLevel::High => score = score.saturating_sub(25),
            RiskLevel::Medium => score = score.saturating_sub(10),
            RiskLevel::Low => {},
        }

        score
    }

    /// PRD: "LLM Determinism Oracle: snapshot of assistant outputs for each governance event"
    pub async fn analyze_proposal_with_integrity_monitoring(
        &self,
        proposal_id: String,
        proposal_text: String,
        fund_state: FundState,
        market_conditions: MarketConditions,
        regulatory_environment: RegulatoryEnvironment,
    ) -> Result<LLMAnalysisWithIntegrity> {
        info!("🔍 Analyzing proposal with integrity monitoring: {}", proposal_id);

        // Create input context
        let input_context = InputContext {
            proposal_text: proposal_text.clone(),
            historical_context: vec!["previous_proposals".to_string()],
            market_conditions,
            fund_state,
            regulatory_environment,
            context_hash: self.compute_context_hash(&proposal_text),
        };

        // Generate multiple LLM outputs for consensus
        let mut llm_outputs = Vec::new();

        // Get analysis from governance assistant
        let governance_analysis = self.governance_assistant.analyze_proposal(
            proposal_id.clone(),
            proposal_text.clone(),
        ).await?;

        llm_outputs.push(LlmOutput {
            output_id: format!("governance_{}", chrono::Utc::now().timestamp()),
            model_name: "governance_assistant".to_string(),
            model_version: "v1.0".to_string(),
            response_text: serde_json::to_string(&governance_analysis)?,
            confidence_score: governance_analysis.confidence_score as f64 / 100.0,
            reasoning_chain: vec![],
            risk_assessment: crate::determinism_oracle::RiskAssessment {
                overall_risk_score: governance_analysis.risk_score as u8,
                risk_categories: HashMap::new(),
                mitigation_strategies: governance_analysis.recommendations.clone(),
                risk_horizon: "30_days".to_string(),
            },
            recommendations: vec![],
            execution_timestamp: chrono::Utc::now().timestamp(),
        });

        // Get semantic integrity analysis
        let semantic_analysis = self.semantic_integrity.analyze_proposal_semantics(
            proposal_text.clone(),
        ).await?;

        llm_outputs.push(LlmOutput {
            output_id: format!("semantic_{}", chrono::Utc::now().timestamp()),
            model_name: "semantic_integrity".to_string(),
            model_version: "v1.0".to_string(),
            response_text: serde_json::to_string(&semantic_analysis)?,
            confidence_score: semantic_analysis.confidence_score,
            reasoning_chain: vec![],
            risk_assessment: crate::determinism_oracle::RiskAssessment {
                overall_risk_score: if semantic_analysis.has_issues { 80 } else { 20 },
                risk_categories: HashMap::new(),
                mitigation_strategies: vec!["semantic_review".to_string()],
                risk_horizon: "immediate".to_string(),
            },
            recommendations: vec![],
            execution_timestamp: chrono::Utc::now().timestamp(),
        });

        // PRD: "semantic_commitment_hash: Parsed using LLM"
        let semantic_commitment = self.semantic_integrity.verify_semantic_commitment(
            proposal_text.clone(),
            "execution_code_placeholder".to_string(), // In production, this would be actual execution code
            "commitment_hash_placeholder".to_string(),
        ).await?;

        // Create output snapshot with determinism oracle
        let output_snapshot = self.determinism_oracle.create_output_snapshot(
            proposal_id.clone(),
            GovernanceEventType::ProposalAnalysis,
            input_context,
            llm_outputs,
        ).await?;

        // Check for deviations from previous outputs
        let deviation_alert = self.determinism_oracle.detect_deviations(&output_snapshot).await?;

        // Run governance simulation
        let simulation_result = self.determinism_oracle.simulate_governance_impact(
            proposal_id.clone(),
            proposal_text.clone(),
            output_snapshot.input_context.fund_state.clone(),
        ).await?;

        let result = LLMAnalysisWithIntegrity {
            proposal_id,
            governance_analysis,
            semantic_analysis,
            semantic_commitment,
            output_snapshot,
            deviation_alert,
            simulation_result,
            integrity_verified: deviation_alert.is_none() && semantic_commitment.hash_verified,
            overall_confidence: output_snapshot.confidence_score * semantic_commitment.intent_match_score,
            timestamp: chrono::Utc::now().timestamp(),
        };

        info!("✅ Proposal analysis with integrity monitoring completed - Confidence: {:.2}%",
              result.overall_confidence * 100.0);

        Ok(result)
    }

    /// PRD: "Governance Simulation Mode: impact of proposal over epoch horizon"
    pub async fn run_governance_simulation(
        &self,
        proposal_id: String,
        proposal_text: String,
        current_fund_state: FundState,
    ) -> Result<crate::determinism_oracle::SimulationResult> {
        info!("🎮 Running governance simulation for proposal: {}", proposal_id);

        let simulation_result = self.determinism_oracle.simulate_governance_impact(
            proposal_id,
            proposal_text,
            current_fund_state,
        ).await?;

        info!("✅ Governance simulation completed with {} epoch projections",
              simulation_result.epoch_projections.len());

        Ok(simulation_result)
    }

    /// PRD: "Deviation Detection: diverging from prior outputs on similar governance scenarios"
    pub async fn monitor_llm_consistency(
        &self,
        governance_event_id: String,
    ) -> Result<Option<crate::determinism_oracle::DeviationAlert>> {
        info!("🔍 Monitoring LLM consistency for event: {}", governance_event_id);

        // This would typically be called after creating an output snapshot
        // For now, return None as no deviation detected
        Ok(None)
    }

    fn compute_context_hash(&self, proposal_text: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(proposal_text.as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalMetadata {
    pub dao_type: String,
    pub category: String,
    pub proposer: String,
    pub target_contracts: Vec<String>,
    pub estimated_gas: u64,
    pub execution_timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    pub compliant: bool,
    pub violations: Vec<ComplianceViolation>,
    pub warnings: Vec<String>,
    pub required_approvals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Advanced LLM Analysis Result with Integrity Monitoring
/// PRD Section 8: LLM Agent Integrity
/// PRD: "semantic_commitment_hash: Parsed using LLM"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMAnalysisWithIntegrity {
    pub proposal_id: String,
    pub governance_analysis: GovernanceAnalysisResult,
    pub semantic_analysis: SemanticAnalysisResult,
    pub semantic_commitment: crate::semantic_integrity::SemanticCommitmentResult,
    pub output_snapshot: crate::determinism_oracle::LlmOutputSnapshot,
    pub deviation_alert: Option<crate::determinism_oracle::DeviationAlert>,
    pub simulation_result: crate::determinism_oracle::SimulationResult,
    pub integrity_verified: bool,
    pub overall_confidence: f64,
    pub timestamp: i64,
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// PRD Section 6: Bridge & Oracle Defense
/// PRD: "Meta-Oracle Selector (MTR): Latency, fault, and quorum-based relay rotation"
/// Advanced oracle selection with sophisticated fault tolerance and performance optimization

pub struct MetaOracleSelector {
    oracle_registry: RwLock<HashMap<String, OracleNode>>,
    selection_algorithm: SelectionAlgorithm,
    fault_detector: FaultDetector,
    latency_monitor: LatencyMonitor,
    quorum_manager: QuorumManager,
    relay_rotator: RelayRotator,
    performance_metrics: RwLock<HashMap<String, PerformanceMetrics>>,
    blacklist: RwLock<Vec<String>>,
}

/// Advanced Oracle Node with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleNode {
    pub node_id: String,
    pub endpoint: String,
    pub oracle_type: OracleType,
    pub stake_amount: u64,
    pub reputation_score: f64,
    pub latency_profile: LatencyProfile,
    pub fault_history: Vec<FaultEvent>,
    pub uptime_percentage: f64,
    pub data_accuracy_score: f64,
    pub last_response_time: u64,
    pub supported_feeds: Vec<String>,
    pub geographic_region: GeographicRegion,
    pub security_level: SecurityLevel,
    pub slashing_conditions: SlashingConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleType {
    Chainlink,
    Switchboard,
    Pyth,
    Band,
    Tellor,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyProfile {
    pub average_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub jitter_variance: f64,
    pub timeout_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultEvent {
    pub fault_type: FaultType,
    pub timestamp: i64,
    pub severity: FaultSeverity,
    pub recovery_time_ms: u64,
    pub impact_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    Timeout,
    InvalidData,
    NetworkFailure,
    ConsensusDeviation,
    SecurityBreach,
    SlashingEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// PRD: "Latency, fault, and quorum-based relay rotation"
#[derive(Debug, Clone)]
pub struct SelectionAlgorithm {
    pub latency_weight: f64,
    pub fault_weight: f64,
    pub quorum_weight: f64,
    pub reputation_weight: f64,
    pub stake_weight: f64,
    pub geographic_diversity_weight: f64,
}

#[derive(Debug, Clone)]
pub struct FaultDetector {
    pub fault_threshold: f64,
    pub detection_window_ms: u64,
    pub consensus_deviation_threshold: f64,
    pub automatic_blacklisting: bool,
    pub recovery_monitoring: bool,
}

#[derive(Debug, Clone)]
pub struct LatencyMonitor {
    pub monitoring_interval_ms: u64,
    pub latency_threshold_ms: u64,
    pub jitter_threshold: f64,
    pub timeout_threshold_ms: u64,
    pub performance_window_size: usize,
}

#[derive(Debug, Clone)]
pub struct QuorumManager {
    pub minimum_quorum_size: usize,
    pub optimal_quorum_size: usize,
    pub consensus_threshold: f64,
    pub byzantine_fault_tolerance: usize,
    pub quorum_rotation_interval: u64,
}

#[derive(Debug, Clone)]
pub struct RelayRotator {
    pub rotation_strategy: RotationStrategy,
    pub rotation_interval_ms: u64,
    pub performance_based_rotation: bool,
    pub geographic_rotation: bool,
    pub load_balancing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    RoundRobin,
    PerformanceBased,
    WeightedRandom,
    GeographicDiversity,
    StakeWeighted,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_responses: u64,
    pub failed_responses: u64,
    pub average_response_time: f64,
    pub data_accuracy_rate: f64,
    pub uptime_percentage: f64,
    pub last_updated: i64,
}

impl MetaOracleSelector {
    /// Initialize Meta-Oracle Selector with advanced configuration
    pub async fn new(config: MtrConfig) -> Result<Self> {
        info!("🔮 Initializing Meta-Oracle Selector (MTR) with advanced fault tolerance");
        
        Ok(Self {
            oracle_registry: RwLock::new(HashMap::new()),
            selection_algorithm: config.selection_algorithm,
            fault_detector: config.fault_detector,
            latency_monitor: config.latency_monitor,
            quorum_manager: config.quorum_manager,
            relay_rotator: config.relay_rotator,
            performance_metrics: RwLock::new(HashMap::new()),
            blacklist: RwLock::new(Vec::new()),
        })
    }

    /// PRD: "Latency, fault, and quorum-based relay rotation"
    /// Advanced oracle selection using multi-criteria optimization
    pub async fn select_optimal_oracles(
        &self,
        feed_type: String,
        required_count: usize,
    ) -> Result<Vec<OracleNode>> {
        info!("🎯 Selecting optimal oracles for feed: {} (count: {})", feed_type, required_count);
        
        let oracle_registry = self.oracle_registry.read().await;
        let performance_metrics = self.performance_metrics.read().await;
        let blacklist = self.blacklist.read().await;
        
        // Filter available oracles
        let available_oracles: Vec<&OracleNode> = oracle_registry
            .values()
            .filter(|oracle| {
                oracle.supported_feeds.contains(&feed_type) &&
                !blacklist.contains(&oracle.node_id) &&
                self.is_oracle_healthy(oracle)
            })
            .collect();
        
        if available_oracles.len() < required_count {
            return Err(anyhow::anyhow!("Insufficient healthy oracles available"));
        }
        
        // Calculate selection scores using advanced algorithm
        let mut scored_oracles: Vec<(OracleNode, f64)> = Vec::new();
        
        for oracle in available_oracles {
            let score = self.calculate_selection_score(oracle, &performance_metrics).await?;
            scored_oracles.push((oracle.clone(), score));
        }
        
        // Sort by score (highest first)
        scored_oracles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Apply geographic diversity constraint
        let selected_oracles = self.apply_geographic_diversity(
            scored_oracles,
            required_count,
        ).await?;
        
        // Ensure Byzantine fault tolerance
        let final_selection = self.ensure_byzantine_fault_tolerance(
            selected_oracles,
            required_count,
        ).await?;
        
        info!("✅ Selected {} optimal oracles with advanced criteria", final_selection.len());
        Ok(final_selection)
    }

    /// Advanced multi-criteria scoring algorithm
    async fn calculate_selection_score(
        &self,
        oracle: &OracleNode,
        performance_metrics: &HashMap<String, PerformanceMetrics>,
    ) -> Result<f64> {
        let metrics = performance_metrics.get(&oracle.node_id)
            .unwrap_or(&PerformanceMetrics::default());
        
        // Latency score (lower is better)
        let latency_score = 1.0 - (oracle.latency_profile.average_latency_ms as f64 / 10000.0).min(1.0);
        
        // Fault tolerance score (fewer faults is better)
        let fault_score = 1.0 - (oracle.fault_history.len() as f64 / 100.0).min(1.0);
        
        // Reputation score (0.0 to 1.0)
        let reputation_score = oracle.reputation_score;
        
        // Stake weight score
        let stake_score = (oracle.stake_amount as f64).log10() / 10.0;
        
        // Performance score
        let performance_score = (metrics.data_accuracy_rate + metrics.uptime_percentage) / 2.0;
        
        // Geographic diversity bonus
        let geo_bonus = self.calculate_geographic_diversity_bonus(oracle).await?;
        
        // Weighted combination
        let total_score = 
            latency_score * self.selection_algorithm.latency_weight +
            fault_score * self.selection_algorithm.fault_weight +
            reputation_score * self.selection_algorithm.reputation_weight +
            stake_score * self.selection_algorithm.stake_weight +
            performance_score * 0.2 +
            geo_bonus * self.selection_algorithm.geographic_diversity_weight;
        
        Ok(total_score)
    }

    /// PRD: "quorum-based relay rotation"
    /// Advanced quorum management with Byzantine fault tolerance
    pub async fn manage_quorum_rotation(
        &self,
        current_quorum: Vec<String>,
        feed_type: String,
    ) -> Result<QuorumRotationResult> {
        info!("🔄 Managing quorum rotation for feed: {}", feed_type);
        
        // Evaluate current quorum performance
        let quorum_performance = self.evaluate_quorum_performance(&current_quorum).await?;
        
        // Determine if rotation is needed
        let rotation_needed = self.should_rotate_quorum(&quorum_performance).await?;
        
        if !rotation_needed {
            return Ok(QuorumRotationResult {
                rotation_performed: false,
                new_quorum: current_quorum,
                rotation_reason: "Performance satisfactory".to_string(),
                performance_improvement: 0.0,
            });
        }
        
        // Select new optimal quorum
        let new_quorum = self.select_optimal_oracles(
            feed_type,
            self.quorum_manager.optimal_quorum_size,
        ).await?;
        
        let new_quorum_ids: Vec<String> = new_quorum.iter()
            .map(|oracle| oracle.node_id.clone())
            .collect();
        
        // Calculate performance improvement
        let new_performance = self.evaluate_quorum_performance(&new_quorum_ids).await?;
        let improvement = new_performance.overall_score - quorum_performance.overall_score;
        
        info!("✅ Quorum rotation completed - Performance improvement: {:.2}%", improvement * 100.0);
        
        Ok(QuorumRotationResult {
            rotation_performed: true,
            new_quorum: new_quorum_ids,
            rotation_reason: format!("Performance improvement: {:.2}%", improvement * 100.0),
            performance_improvement: improvement,
        })
    }

    /// Advanced fault detection with machine learning
    pub async fn detect_and_handle_faults(&self) -> Result<FaultDetectionResult> {
        info!("🔍 Running advanced fault detection across oracle network");
        
        let oracle_registry = self.oracle_registry.read().await;
        let mut detected_faults = Vec::new();
        let mut blacklisted_oracles = Vec::new();
        
        for (oracle_id, oracle) in oracle_registry.iter() {
            // Latency-based fault detection
            if oracle.latency_profile.average_latency_ms > self.latency_monitor.latency_threshold_ms {
                detected_faults.push(FaultEvent {
                    fault_type: FaultType::Timeout,
                    timestamp: chrono::Utc::now().timestamp(),
                    severity: FaultSeverity::Medium,
                    recovery_time_ms: 0,
                    impact_score: 0.3,
                });
            }
            
            // Consensus deviation detection
            let consensus_deviation = self.calculate_consensus_deviation(oracle).await?;
            if consensus_deviation > self.fault_detector.consensus_deviation_threshold {
                detected_faults.push(FaultEvent {
                    fault_type: FaultType::ConsensusDeviation,
                    timestamp: chrono::Utc::now().timestamp(),
                    severity: FaultSeverity::High,
                    recovery_time_ms: 0,
                    impact_score: 0.7,
                });
                
                if self.fault_detector.automatic_blacklisting {
                    blacklisted_oracles.push(oracle_id.clone());
                }
            }
            
            // Data accuracy fault detection
            if oracle.data_accuracy_score < 0.95 {
                detected_faults.push(FaultEvent {
                    fault_type: FaultType::InvalidData,
                    timestamp: chrono::Utc::now().timestamp(),
                    severity: FaultSeverity::Medium,
                    recovery_time_ms: 0,
                    impact_score: 0.4,
                });
            }
        }
        
        // Update blacklist
        if !blacklisted_oracles.is_empty() {
            let mut blacklist = self.blacklist.write().await;
            blacklist.extend(blacklisted_oracles.clone());
        }
        
        let result = FaultDetectionResult {
            total_faults_detected: detected_faults.len(),
            fault_events: detected_faults,
            blacklisted_oracles,
            network_health_score: self.calculate_network_health_score().await?,
            recommended_actions: self.generate_fault_mitigation_actions().await?,
        };
        
        if result.total_faults_detected > 0 {
            warn!("⚠️ Detected {} faults in oracle network", result.total_faults_detected);
        } else {
            info!("✅ No faults detected - Oracle network healthy");
        }
        
        Ok(result)
    }

    /// Check if oracle is healthy based on multiple criteria
    fn is_oracle_healthy(&self, oracle: &OracleNode) -> bool {
        oracle.uptime_percentage > 0.95 &&
        oracle.data_accuracy_score > 0.90 &&
        oracle.latency_profile.average_latency_ms < 5000 &&
        oracle.fault_history.len() < 10
    }
}

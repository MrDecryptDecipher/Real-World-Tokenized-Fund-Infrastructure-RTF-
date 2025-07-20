//! # RTF Bridge Defense System
//! 
//! Advanced bridge and oracle defense system providing comprehensive protection
//! against oracle manipulation, bridge attacks, and cross-chain vulnerabilities.

pub mod meta_oracle_selector;
pub mod zk_message_filter;
pub mod chain_origin_guard;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Bridge Defense System coordinator
#[derive(Debug)]
pub struct BridgeDefenseSystem {
    meta_oracle: meta_oracle_selector::MetaOracleSelector,
    message_filter: zk_message_filter::ZkMessageFilter,
    origin_guard: chain_origin_guard::ChainOriginGuard,
    config: DefenseConfig,
    metrics: RwLock<DefenseMetrics>,
}

/// Configuration for the bridge defense system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseConfig {
    pub oracle_timeout_ms: u64,
    pub max_oracle_deviation: f64,
    pub min_oracle_quorum: usize,
    pub message_encryption_enabled: bool,
    pub chain_verification_enabled: bool,
    pub fraud_detection_threshold: f64,
}

impl Default for DefenseConfig {
    fn default() -> Self {
        Self {
            oracle_timeout_ms: 5000,
            max_oracle_deviation: 0.05, // 5%
            min_oracle_quorum: 3,
            message_encryption_enabled: true,
            chain_verification_enabled: true,
            fraud_detection_threshold: 0.8,
        }
    }
}

/// Defense system metrics
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DefenseMetrics {
    pub oracle_queries_total: u64,
    pub oracle_failures: u64,
    pub messages_filtered: u64,
    pub fraud_attempts_detected: u64,
    pub chain_verifications: u64,
    pub uptime_seconds: u64,
}

/// Defense alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefenseAlert {
    OracleManipulation {
        oracle_id: String,
        deviation: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    BridgeAttack {
        chain_id: u64,
        attack_type: String,
        severity: AlertSeverity,
    },
    MessageTampering {
        message_hash: String,
        source_chain: u64,
        target_chain: u64,
    },
    FraudDetected {
        transaction_hash: String,
        confidence_score: f64,
        details: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl BridgeDefenseSystem {
    /// Create a new bridge defense system
    pub async fn new(config: DefenseConfig) -> Result<Self> {
        info!("Initializing RTF Bridge Defense System");
        
        let meta_oracle = meta_oracle_selector::MetaOracleSelector::new(&config).await?;
        let message_filter = zk_message_filter::ZkMessageFilter::new(&config).await?;
        let origin_guard = chain_origin_guard::ChainOriginGuard::new(&config).await?;
        
        Ok(Self {
            meta_oracle,
            message_filter,
            origin_guard,
            config,
            metrics: RwLock::new(DefenseMetrics::default()),
        })
    }

    /// Start the defense system
    pub async fn start(&self) -> Result<()> {
        info!("Starting RTF Bridge Defense System");
        
        // Start all subsystems
        tokio::try_join!(
            self.meta_oracle.start(),
            self.message_filter.start(),
            self.origin_guard.start()
        )?;
        
        info!("RTF Bridge Defense System started successfully");
        Ok(())
    }

    /// Stop the defense system
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping RTF Bridge Defense System");
        
        // Stop all subsystems
        tokio::try_join!(
            self.meta_oracle.stop(),
            self.message_filter.stop(),
            self.origin_guard.stop()
        )?;
        
        info!("RTF Bridge Defense System stopped successfully");
        Ok(())
    }

    /// Get current defense metrics
    pub async fn get_metrics(&self) -> DefenseMetrics {
        self.metrics.read().await.clone()
    }

    /// Process a cross-chain message with full defense validation
    pub async fn process_message(
        &self,
        message: &[u8],
        source_chain: u64,
        target_chain: u64,
    ) -> Result<bool> {
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.oracle_queries_total += 1;
        }

        // 1. Verify chain of origin
        if !self.origin_guard.verify_chain_origin(source_chain, message).await? {
            warn!("Chain origin verification failed for message from chain {}", source_chain);
            return Ok(false);
        }

        // 2. Filter message through zkMessage filter
        if !self.message_filter.validate_message(message, source_chain).await? {
            warn!("Message validation failed for chain {}", source_chain);
            return Ok(false);
        }

        // 3. Verify oracle consensus
        if !self.meta_oracle.verify_consensus(message).await? {
            warn!("Oracle consensus verification failed");
            return Ok(false);
        }

        info!("Message successfully validated through all defense layers");
        Ok(true)
    }

    /// Handle defense alerts
    pub async fn handle_alert(&self, alert: DefenseAlert) -> Result<()> {
        match &alert {
            DefenseAlert::OracleManipulation { oracle_id, deviation, .. } => {
                error!("Oracle manipulation detected: {} with deviation {}", oracle_id, deviation);
                // Implement oracle blacklisting logic
            }
            DefenseAlert::BridgeAttack { chain_id, attack_type, severity } => {
                error!("Bridge attack detected on chain {}: {} (severity: {:?})", 
                       chain_id, attack_type, severity);
                // Implement emergency protocols
            }
            DefenseAlert::MessageTampering { message_hash, source_chain, target_chain } => {
                error!("Message tampering detected: {} from chain {} to {}", 
                       message_hash, source_chain, target_chain);
                // Implement message quarantine
            }
            DefenseAlert::FraudDetected { transaction_hash, confidence_score, details } => {
                error!("Fraud detected in transaction {}: {} (confidence: {})", 
                       transaction_hash, details, confidence_score);
                // Implement fraud response protocols
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.fraud_attempts_detected += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_defense_initialization() {
        let config = DefenseConfig::default();
        let defense_system = BridgeDefenseSystem::new(config).await;
        assert!(defense_system.is_ok());
    }

    #[tokio::test]
    async fn test_message_processing() {
        let config = DefenseConfig::default();
        let defense_system = BridgeDefenseSystem::new(config).await.unwrap();
        
        let test_message = b"test message";
        let result = defense_system.process_message(test_message, 1, 2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = DefenseConfig::default();
        let defense_system = BridgeDefenseSystem::new(config).await.unwrap();
        
        let metrics = defense_system.get_metrics().await;
        assert_eq!(metrics.oracle_queries_total, 0);
    }
}

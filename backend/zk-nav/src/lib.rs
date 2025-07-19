pub mod zkreplay_integrity;

pub use zkreplay_integrity::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF zkNAV Service Library
/// PRD Section 5: "zkReplay & Integrity System"
/// PRD: "Triple-check replay roots: Ethereum, Solana, BTC anchor"
/// PRD: "Drift ledger: Tracks root Δ across epochs"
/// PRD: "Deviation > threshold = redemption freeze"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkNavConfig {
    pub deviation_threshold: f64,
    pub freeze_threshold: f64,
    pub epoch_duration: u64,
    pub integrity_enabled: bool,
}

impl Default for ZkNavConfig {
    fn default() -> Self {
        Self {
            deviation_threshold: 0.05, // 5% deviation threshold
            freeze_threshold: 0.10,    // 10% freeze threshold
            epoch_duration: 86400,     // 24 hours
            integrity_enabled: true,
        }
    }
}

/// Initialize zkNAV service with integrity system
pub async fn init_zknav_service(config: ZkNavConfig) -> Result<ZkReplayIntegritySystem> {
    info!("🔢 Initializing RTF zkNAV Service with Integrity System");

    let integrity_system = ZkReplayIntegritySystem::new(
        config.deviation_threshold,
        config.freeze_threshold,
        config.epoch_duration,
    ).await?;

    info!("✅ RTF zkNAV Service initialized successfully");
    Ok(integrity_system)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zknav_service_initialization() {
        let config = ZkNavConfig::default();
        let result = init_zknav_service(config).await;
        assert!(result.is_ok());
    }
}

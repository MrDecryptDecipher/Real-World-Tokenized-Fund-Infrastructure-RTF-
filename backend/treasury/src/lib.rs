pub mod ai_treasury_service;

pub use ai_treasury_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF Treasury Management Service Library
/// Provides AI-powered treasury management and portfolio optimization

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryConfig {
    pub ai_enabled: bool,
    pub max_position_size: f64,
    pub rebalancing_enabled: bool,
    pub risk_tolerance: String,
}

impl Default for TreasuryConfig {
    fn default() -> Self {
        Self {
            ai_enabled: true,
            max_position_size: 0.25, // 25% max position
            rebalancing_enabled: true,
            risk_tolerance: "moderate".to_string(),
        }
    }
}

/// Initialize treasury service
pub async fn init_treasury_service(config: TreasuryConfig) -> Result<AITreasuryService> {
    info!("💰 Initializing RTF Treasury Management Service");

    let risk_tolerance = match config.risk_tolerance.as_str() {
        "conservative" => RiskTolerance::Conservative,
        "aggressive" => RiskTolerance::Aggressive,
        "dynamic" => RiskTolerance::Dynamic,
        _ => RiskTolerance::Moderate,
    };

    let service = AITreasuryService::new(
        risk_tolerance,
        config.max_position_size,
    ).await?;

    info!("✅ RTF Treasury Management Service initialized successfully");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_treasury_service_initialization() {
        let config = TreasuryConfig::default();
        let result = init_treasury_service(config).await;
        assert!(result.is_ok());
    }
}

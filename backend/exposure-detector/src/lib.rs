pub mod fund_exposure_service;

pub use fund_exposure_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF Fund Exposure Detection Service Library
/// Provides fund exposure analysis and circular dependency detection

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureConfig {
    pub max_exposure_depth: usize,
    pub max_circular_exposure: f64,
    pub monitoring_enabled: bool,
    pub real_time_analysis: bool,
}

impl Default for ExposureConfig {
    fn default() -> Self {
        Self {
            max_exposure_depth: 5,
            max_circular_exposure: 0.25, // 25% max circular exposure
            monitoring_enabled: true,
            real_time_analysis: true,
        }
    }
}

/// Initialize exposure detection service
pub async fn init_exposure_service(config: ExposureConfig) -> Result<FundExposureService> {
    info!("🕸️ Initializing RTF Fund Exposure Detection Service");

    let service = FundExposureService::new(
        config.max_exposure_depth,
        config.max_circular_exposure,
    ).await?;

    info!("✅ RTF Fund Exposure Detection Service initialized successfully");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exposure_service_initialization() {
        let config = ExposureConfig::default();
        let result = init_exposure_service(config).await;
        assert!(result.is_ok());
    }
}

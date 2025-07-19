pub mod metrics_service;

pub use metrics_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF Monitoring Service Library
/// Provides comprehensive monitoring and metrics collection for RTF Infrastructure

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_retention_days: u32,
    pub alert_check_interval_seconds: u64,
    pub performance_threshold_ms: u64,
    pub enabled: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_retention_days: 30,
            alert_check_interval_seconds: 60,
            performance_threshold_ms: 700, // PRD requirement
            enabled: true,
        }
    }
}

/// Initialize monitoring service
pub async fn init_monitoring(config: MonitoringConfig) -> Result<MetricsService> {
    info!("🔧 Initializing RTF Monitoring Service");
    
    let service = MetricsService::new().await?;
    
    info!("✅ RTF Monitoring Service initialized successfully");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_initialization() {
        let config = MonitoringConfig::default();
        let result = init_monitoring(config).await;
        assert!(result.is_ok());
    }
}

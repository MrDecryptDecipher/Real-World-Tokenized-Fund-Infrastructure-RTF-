pub mod zk_kyc_service;

pub use zk_kyc_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF Compliance Service Library
/// Provides zk-KYC and regulatory compliance functionality

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub zk_kyc_enabled: bool,
    pub supported_jurisdictions: Vec<String>,
    pub kilt_endpoint: String,
    pub fractal_endpoint: String,
    pub worldid_app_id: String,
    pub sismo_group_id: String,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            zk_kyc_enabled: true,
            supported_jurisdictions: vec!["US".to_string(), "EU".to_string()],
            kilt_endpoint: "https://api.kilt.io".to_string(),
            fractal_endpoint: "https://api.fractal.id".to_string(),
            worldid_app_id: "app_rtf_worldid".to_string(),
            sismo_group_id: "rtf_verified_users".to_string(),
        }
    }
}

/// Initialize compliance service
pub async fn init_compliance_service(config: ComplianceConfig) -> Result<ZkKycService> {
    info!("🔐 Initializing RTF Compliance Service");

    let service = ZkKycService::new_with_providers(
        config.kilt_endpoint,
        config.fractal_endpoint,
        config.worldid_app_id,
        config.sismo_group_id,
    ).await?;

    info!("✅ RTF Compliance Service initialized successfully");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compliance_service_initialization() {
        let config = ComplianceConfig::default();
        let result = init_compliance_service(config).await;
        assert!(result.is_ok());
    }
}

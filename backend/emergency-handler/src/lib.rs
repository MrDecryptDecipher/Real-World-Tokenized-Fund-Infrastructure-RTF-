pub mod emergency_service;

pub use emergency_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// RTF Emergency Handler Service Library
/// Provides emergency response and circuit breaker functionality

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyConfig {
    pub circuit_breaker_enabled: bool,
    pub suicide_lock_delay_hours: u64,
    pub auto_response_enabled: bool,
    pub emergency_contacts: Vec<String>,
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            circuit_breaker_enabled: true,
            suicide_lock_delay_hours: 168, // 7 days
            auto_response_enabled: true,
            emergency_contacts: vec!["emergency@rtf.finance".to_string()],
        }
    }
}

/// Initialize emergency service
pub async fn init_emergency_service(config: EmergencyConfig) -> Result<EmergencyService> {
    info!("🚨 Initializing RTF Emergency Handler Service");

    let emergency_contacts = config.emergency_contacts.iter().map(|email| EmergencyContact {
        name: "Emergency Team".to_string(),
        role: "Emergency Response".to_string(),
        email: email.clone(),
        phone: "+1-555-EMERGENCY".to_string(),
        telegram: None,
        priority: 1,
        available_24_7: true,
    }).collect();

    let service = EmergencyService::new(
        "emergency_multisig".to_string(),
        emergency_contacts,
    ).await?;

    info!("✅ RTF Emergency Handler Service initialized successfully");
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_emergency_service_initialization() {
        let config = EmergencyConfig::default();
        let result = init_emergency_service(config).await;
        assert!(result.is_ok());
    }
}

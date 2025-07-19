pub mod ccip_service;
pub mod babylon_service;
pub mod icp_service;
pub mod cross_chain_coordinator;
pub mod celestia_service;
pub mod zknav_cross_chain;

pub use ccip_service::*;
pub use babylon_service::*;
pub use icp_service::*;
pub use cross_chain_coordinator::*;
pub use celestia_service::*;
pub use zknav_cross_chain::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Cross-Chain zkNAV Integration Service
/// PRD Section 3.2: "Posted to Solana, Anchored to BTC via Babylon, Pushed to Ethereum via CCIP, Stored in Celestia"
/// PRD: "NAV is computed daily using a verifiable zk circuit"
/// PRD: "zkNAV drift ledger over 100 epochs"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainNavConfig {
    pub solana_rpc_url: String,
    pub babylon_rpc_url: String,
    pub ethereum_rpc_url: String,
    pub celestia_rpc_url: String,
    pub icp_gateway_url: String,
    pub drift_threshold: f64,
    pub verification_timeout_seconds: u64,
}

impl Default for CrossChainNavConfig {
    fn default() -> Self {
        Self {
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            babylon_rpc_url: "https://rpc.babylon.network".to_string(),
            ethereum_rpc_url: "https://eth-mainnet.g.alchemy.com/v2/api-key".to_string(),
            celestia_rpc_url: "https://rpc.celestia.network".to_string(),
            icp_gateway_url: "https://ic0.app".to_string(),
            drift_threshold: 0.05, // 5%
            verification_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Initialize cross-chain zkNAV service
pub async fn init_cross_chain_zknav(config: CrossChainNavConfig) -> Result<CrossChainZkNavService> {
    info!("🌐 Initializing Cross-Chain zkNAV Service");

    let service = CrossChainZkNavService::new(
        config.solana_rpc_url,
        config.babylon_rpc_url,
        config.ethereum_rpc_url,
        config.celestia_rpc_url,
        config.icp_gateway_url,
    ).await?;

    info!("✅ Cross-Chain zkNAV Service initialized successfully");
    Ok(service)
}
pub mod filecoin_service;

pub use ccip_service::*;
pub use babylon_service::*;
pub use icp_service::*;
pub use cross_chain_coordinator::*;
pub use celestia_service::*;
pub use filecoin_service::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, error};

/// Comprehensive Cross-Chain Service for RTF Infrastructure
/// July 2025 - Multi-chain coordination with latest technologies
pub struct CrossChainService {
    ccip_service: CCIPService,
    babylon_service: BabylonService,
    icp_service: ICPService,
    coordinator: CrossChainCoordinator,
    supported_chains: RwLock<HashMap<u64, ChainInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub vault_address: String,
    pub status: ChainStatus,
    pub last_sync: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainStatus {
    Active,
    Syncing,
    Degraded,
    Offline,
}

impl CrossChainService {
    /// Initialize comprehensive cross-chain service with July 2025 technologies
    pub async fn new_with_ccip(
        ccip_config: CCIPConfig,
        babylon_config: BabylonConfig,
        icp_config: ICPConfig,
    ) -> Result<Self> {
        info!("🌐 Initializing Comprehensive Cross-Chain Service");

        // Initialize individual services
        let ccip_service = CCIPService::new_with_svm_support(
            ccip_config.router_address,
            ccip_config.token_pool_address,
            ccip_config.chain_configs,
        ).await?;

        let babylon_service = BabylonService::new_phase2_mainnet(
            babylon_config.babylon_rpc_url,
            babylon_config.bitcoin_rpc_url,
            babylon_config.finality_provider_address,
        ).await?;

        let icp_service = ICPService::new_with_chain_fusion(
            icp_config.replica_url,
            icp_config.canister_id,
            icp_config.identity_path,
        ).await?;

        let coordinator = CrossChainCoordinator::new().await?;

        let service = Self {
            ccip_service,
            babylon_service,
            icp_service,
            coordinator,
            supported_chains: RwLock::new(HashMap::new()),
        };

        // Initialize supported chains
        service.initialize_supported_chains().await?;

        info!("✅ Cross-Chain Service initialized with all protocols");
        Ok(service)
    }

    /// Synchronize vault state across all supported chains
    pub async fn sync_vault_state_comprehensive(
        &self,
        vault_id: String,
        nav_data: NavData,
    ) -> Result<CrossChainSyncResult> {
        info!("🔄 Starting comprehensive vault synchronization for {}", vault_id);

        let mut sync_result = CrossChainSyncResult {
            vault_id: vault_id.clone(),
            successful_chains: Vec::new(),
            failed_chains: Vec::new(),
            bitcoin_anchor: None,
            icp_verification: None,
            total_time_ms: 0,
        };

        let start_time = std::time::Instant::now();

        // 1. Anchor to Bitcoin via Babylon
        match self.babylon_service.anchor_vault_nav_to_bitcoin(
            vault_id.clone(),
            nav_data.computation_hash.clone(),
            nav_data.epoch,
        ).await {
            Ok(anchor) => {
                sync_result.bitcoin_anchor = Some(anchor);
                info!("✅ Bitcoin anchor successful");
            },
            Err(e) => {
                error!("❌ Bitcoin anchor failed: {}", e);
                sync_result.failed_chains.push("Bitcoin".to_string());
            }
        }

        // 2. Sync via CCIP to supported chains
        let chains = self.supported_chains.read().await;
        let chain_ids: Vec<u64> = chains.keys().cloned().collect();
        drop(chains);

        if !chain_ids.is_empty() {
            let vault_sync = CrossChainVaultSync {
                vault_id: vault_id.clone(),
                source_chain: 1, // Ethereum as source
                destination_chains: chain_ids.clone(),
                nav_data: nav_data.clone(),
                proof_hash: nav_data.computation_hash.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };

            match self.ccip_service.sync_vault_cross_chain(vault_sync).await {
                Ok(message_ids) => {
                    for (i, chain_id) in chain_ids.iter().enumerate() {
                        sync_result.successful_chains.push(format!("Chain-{}", chain_id));
                        info!("✅ CCIP sync successful for chain {} with message {}",
                              chain_id, message_ids.get(i).unwrap_or(&"unknown".to_string()));
                    }
                },
                Err(e) => {
                    error!("❌ CCIP sync failed: {}", e);
                    for chain_id in &chain_ids {
                        sync_result.failed_chains.push(format!("Chain-{}", chain_id));
                    }
                }
            }
        }

        // 3. Verify via ICP Chain Fusion
        match self.icp_service.verify_cross_chain_state(
            vault_id.clone(),
            nav_data.clone(),
        ).await {
            Ok(verification) => {
                sync_result.icp_verification = Some(verification);
                info!("✅ ICP verification successful");
            },
            Err(e) => {
                error!("❌ ICP verification failed: {}", e);
                sync_result.failed_chains.push("ICP".to_string());
            }
        }

        sync_result.total_time_ms = start_time.elapsed().as_millis() as u64;

        info!("🏁 Cross-chain sync completed in {}ms - Success: {}, Failed: {}",
              sync_result.total_time_ms,
              sync_result.successful_chains.len(),
              sync_result.failed_chains.len());

        Ok(sync_result)
    }

    async fn initialize_supported_chains(&self) -> Result<()> {
        let mut chains = self.supported_chains.write().await;

        // Ethereum
        chains.insert(1, ChainInfo {
            chain_id: 1,
            name: "Ethereum".to_string(),
            rpc_url: "https://eth-mainnet.g.alchemy.com/v2/key".to_string(),
            vault_address: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4Db44".to_string(),
            status: ChainStatus::Active,
            last_sync: 0,
        });

        // Avalanche
        chains.insert(43114, ChainInfo {
            chain_id: 43114,
            name: "Avalanche".to_string(),
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            vault_address: "0x742d35Cc6634C0532925a3b8D4C9db96C4b4Db44".to_string(),
            status: ChainStatus::Active,
            last_sync: 0,
        });

        // Solana (represented as chain for CCIP)
        chains.insert(999999, ChainInfo {
            chain_id: 999999,
            name: "Solana".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            vault_address: "RTFVau1tAdvancedSPLTokenVau1tProgram11111111".to_string(),
            status: ChainStatus::Active,
            last_sync: 0,
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainSyncResult {
    pub vault_id: String,
    pub successful_chains: Vec<String>,
    pub failed_chains: Vec<String>,
    pub bitcoin_anchor: Option<BitcoinAnchor>,
    pub icp_verification: Option<ICPVerification>,
    pub total_time_ms: u64,
}

// Configuration structures
#[derive(Debug, Clone)]
pub struct CCIPConfig {
    pub router_address: String,
    pub token_pool_address: String,
    pub chain_configs: HashMap<u64, ChainConfig>,
}

#[derive(Debug, Clone)]
pub struct BabylonConfig {
    pub babylon_rpc_url: String,
    pub bitcoin_rpc_url: String,
    pub finality_provider_address: String,
}

#[derive(Debug, Clone)]
pub struct ICPConfig {
    pub replica_url: String,
    pub canister_id: String,
    pub identity_path: String,
}

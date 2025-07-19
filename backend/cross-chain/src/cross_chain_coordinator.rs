use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep, Instant};
use tracing::{info, warn, error};

/// Cross-Chain Coordinator for RTF Infrastructure
/// Orchestrates multi-chain operations with fault tolerance and recovery
pub struct CrossChainCoordinator {
    active_operations: RwLock<HashMap<String, CrossChainOperation>>,
    chain_health: RwLock<HashMap<u64, ChainHealth>>,
    operation_timeout: Duration,
    max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainOperation {
    pub operation_id: String,
    pub vault_id: String,
    pub operation_type: OperationType,
    pub target_chains: Vec<u64>,
    pub status: OperationStatus,
    pub started_at: i64,
    pub completed_chains: Vec<u64>,
    pub failed_chains: Vec<u64>,
    pub retry_count: u32,
    pub error_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    NavSync,
    VaultCreation,
    EmergencyPause,
    GovernanceUpdate,
    ComplianceVerification,
    LiquidityRebalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationStatus {
    Pending,
    InProgress,
    PartialSuccess,
    Success,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainHealth {
    pub chain_id: u64,
    pub status: ChainHealthStatus,
    pub last_successful_operation: i64,
    pub consecutive_failures: u32,
    pub average_response_time_ms: u64,
    pub last_health_check: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainHealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub operation_id: String,
    pub success: bool,
    pub successful_chains: Vec<u64>,
    pub failed_chains: Vec<u64>,
    pub total_time_ms: u64,
    pub retry_attempts: u32,
}

impl CrossChainCoordinator {
    /// Initialize cross-chain coordinator
    pub async fn new() -> Result<Self> {
        info!("🎯 Initializing Cross-Chain Coordinator");

        let coordinator = Self {
            active_operations: RwLock::new(HashMap::new()),
            chain_health: RwLock::new(HashMap::new()),
            operation_timeout: Duration::from_secs(300), // 5 minutes
            max_retries: 3,
        };

        // Initialize chain health monitoring
        coordinator.initialize_chain_health_monitoring().await?;

        info!("✅ Cross-Chain Coordinator initialized");
        Ok(coordinator)
    }

    /// Coordinate cross-chain operation with fault tolerance
    pub async fn coordinate_operation(
        &self,
        vault_id: String,
        operation_type: OperationType,
        target_chains: Vec<u64>,
    ) -> Result<CoordinationResult> {
        let operation_id = format!("op_{}_{}", 
                                 chrono::Utc::now().timestamp(), 
                                 uuid::Uuid::new_v4().to_string()[..8].to_string());

        info!("🚀 Starting cross-chain operation {} for vault {}", operation_id, vault_id);

        let start_time = Instant::now();

        // Create operation record
        let mut operation = CrossChainOperation {
            operation_id: operation_id.clone(),
            vault_id: vault_id.clone(),
            operation_type: operation_type.clone(),
            target_chains: target_chains.clone(),
            status: OperationStatus::Pending,
            started_at: chrono::Utc::now().timestamp(),
            completed_chains: Vec::new(),
            failed_chains: Vec::new(),
            retry_count: 0,
            error_messages: Vec::new(),
        };

        // Register operation
        {
            let mut operations = self.active_operations.write().await;
            operations.insert(operation_id.clone(), operation.clone());
        }

        // Filter healthy chains
        let healthy_chains = self.filter_healthy_chains(&target_chains).await?;
        if healthy_chains.is_empty() {
            error!("❌ No healthy chains available for operation");
            operation.status = OperationStatus::Failed;
            operation.error_messages.push("No healthy chains available".to_string());
            return Ok(self.create_coordination_result(&operation, start_time));
        }

        // Execute operation with retries
        operation.status = OperationStatus::InProgress;
        self.update_operation(&operation).await?;

        let mut current_chains = healthy_chains;
        
        while operation.retry_count <= self.max_retries && !current_chains.is_empty() {
            info!("🔄 Attempt {} for operation {} on {} chains", 
                  operation.retry_count + 1, operation_id, current_chains.len());

            let batch_result = self.execute_operation_batch(
                &operation,
                &current_chains,
            ).await?;

            // Update operation status
            operation.completed_chains.extend(batch_result.successful_chains.clone());
            operation.failed_chains.extend(batch_result.failed_chains.clone());
            operation.retry_count += 1;

            // Remove successful chains from retry list
            current_chains.retain(|&chain_id| !batch_result.successful_chains.contains(&chain_id));

            // Check if operation is complete
            if current_chains.is_empty() {
                operation.status = OperationStatus::Success;
                break;
            } else if operation.retry_count > self.max_retries {
                operation.status = OperationStatus::Failed;
                break;
            } else {
                operation.status = OperationStatus::PartialSuccess;
                
                // Wait before retry with exponential backoff
                let backoff_duration = Duration::from_secs(2_u64.pow(operation.retry_count));
                warn!("⏳ Retrying operation {} in {:?}", operation_id, backoff_duration);
                sleep(backoff_duration).await;
            }

            self.update_operation(&operation).await?;
        }

        // Final status determination
        if operation.completed_chains.len() == target_chains.len() {
            operation.status = OperationStatus::Success;
        } else if !operation.completed_chains.is_empty() {
            operation.status = OperationStatus::PartialSuccess;
        } else {
            operation.status = OperationStatus::Failed;
        }

        self.update_operation(&operation).await?;

        let result = self.create_coordination_result(&operation, start_time);

        info!("🏁 Operation {} completed with status {:?} in {}ms", 
              operation_id, operation.status, result.total_time_ms);

        // Clean up operation record
        {
            let mut operations = self.active_operations.write().await;
            operations.remove(&operation_id);
        }

        Ok(result)
    }

    /// Monitor chain health continuously
    pub async fn monitor_chain_health(&self) -> Result<()> {
        info!("💓 Starting continuous chain health monitoring");

        loop {
            let chains = {
                let health = self.chain_health.read().await;
                health.keys().cloned().collect::<Vec<_>>()
            };

            for chain_id in chains {
                if let Err(e) = self.check_chain_health(chain_id).await {
                    error!("❌ Health check failed for chain {}: {}", chain_id, e);
                }
            }

            // Sleep for 30 seconds between health checks
            sleep(Duration::from_secs(30)).await;
        }
    }

    /// Get operation status
    pub async fn get_operation_status(&self, operation_id: &str) -> Result<Option<CrossChainOperation>> {
        let operations = self.active_operations.read().await;
        Ok(operations.get(operation_id).cloned())
    }

    /// Get chain health status
    pub async fn get_chain_health_status(&self) -> Result<HashMap<u64, ChainHealth>> {
        let health = self.chain_health.read().await;
        Ok(health.clone())
    }

    // Private helper methods
    async fn initialize_chain_health_monitoring(&self) -> Result<()> {
        let mut health = self.chain_health.write().await;
        
        // Initialize health for supported chains
        let supported_chains = vec![1, 43114, 999999]; // Ethereum, Avalanche, Solana
        
        for chain_id in supported_chains {
            health.insert(chain_id, ChainHealth {
                chain_id,
                status: ChainHealthStatus::Healthy,
                last_successful_operation: chrono::Utc::now().timestamp(),
                consecutive_failures: 0,
                average_response_time_ms: 0,
                last_health_check: chrono::Utc::now().timestamp(),
            });
        }

        Ok(())
    }

    async fn filter_healthy_chains(&self, target_chains: &[u64]) -> Result<Vec<u64>> {
        let health = self.chain_health.read().await;
        
        let healthy_chains: Vec<u64> = target_chains
            .iter()
            .filter(|&&chain_id| {
                health.get(&chain_id)
                    .map(|h| matches!(h.status, ChainHealthStatus::Healthy | ChainHealthStatus::Degraded))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        Ok(healthy_chains)
    }

    async fn execute_operation_batch(
        &self,
        operation: &CrossChainOperation,
        chains: &[u64],
    ) -> Result<BatchResult> {
        info!("⚡ Executing operation batch on {} chains", chains.len());

        let mut successful_chains = Vec::new();
        let mut failed_chains = Vec::new();

        // Execute operation on each chain concurrently
        let mut tasks = Vec::new();
        
        for &chain_id in chains {
            let operation_clone = operation.clone();
            let task = tokio::spawn(async move {
                Self::execute_single_chain_operation(chain_id, operation_clone).await
            });
            tasks.push((chain_id, task));
        }

        // Wait for all tasks to complete
        for (chain_id, task) in tasks {
            match task.await {
                Ok(Ok(_)) => {
                    successful_chains.push(chain_id);
                    self.update_chain_health_success(chain_id).await?;
                },
                Ok(Err(e)) => {
                    failed_chains.push(chain_id);
                    self.update_chain_health_failure(chain_id, &e.to_string()).await?;
                    error!("❌ Operation failed on chain {}: {}", chain_id, e);
                },
                Err(e) => {
                    failed_chains.push(chain_id);
                    self.update_chain_health_failure(chain_id, &e.to_string()).await?;
                    error!("❌ Task failed for chain {}: {}", chain_id, e);
                }
            }
        }

        Ok(BatchResult {
            successful_chains,
            failed_chains,
        })
    }

    async fn execute_single_chain_operation(
        chain_id: u64,
        operation: CrossChainOperation,
    ) -> Result<()> {
        info!("🔗 Executing operation {} on chain {}", operation.operation_id, chain_id);

        // Simulate operation execution based on type
        match operation.operation_type {
            OperationType::NavSync => {
                // Simulate NAV sync
                sleep(Duration::from_millis(100)).await;
            },
            OperationType::VaultCreation => {
                // Simulate vault creation
                sleep(Duration::from_millis(500)).await;
            },
            OperationType::EmergencyPause => {
                // Simulate emergency pause
                sleep(Duration::from_millis(50)).await;
            },
            _ => {
                // Default operation
                sleep(Duration::from_millis(200)).await;
            }
        }

        // Simulate occasional failures for testing
        if chain_id == 999999 && operation.retry_count == 0 {
            return Err(anyhow::anyhow!("Simulated failure for testing"));
        }

        Ok(())
    }

    async fn update_operation(&self, operation: &CrossChainOperation) -> Result<()> {
        let mut operations = self.active_operations.write().await;
        operations.insert(operation.operation_id.clone(), operation.clone());
        Ok(())
    }

    async fn check_chain_health(&self, chain_id: u64) -> Result<()> {
        let start_time = Instant::now();
        
        // Simulate health check
        sleep(Duration::from_millis(50)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        // Update health record
        let mut health = self.chain_health.write().await;
        if let Some(chain_health) = health.get_mut(&chain_id) {
            chain_health.last_health_check = chrono::Utc::now().timestamp();
            chain_health.average_response_time_ms = 
                (chain_health.average_response_time_ms + response_time) / 2;
        }

        Ok(())
    }

    async fn update_chain_health_success(&self, chain_id: u64) -> Result<()> {
        let mut health = self.chain_health.write().await;
        if let Some(chain_health) = health.get_mut(&chain_id) {
            chain_health.last_successful_operation = chrono::Utc::now().timestamp();
            chain_health.consecutive_failures = 0;
            chain_health.status = ChainHealthStatus::Healthy;
        }
        Ok(())
    }

    async fn update_chain_health_failure(&self, chain_id: u64, error: &str) -> Result<()> {
        let mut health = self.chain_health.write().await;
        if let Some(chain_health) = health.get_mut(&chain_id) {
            chain_health.consecutive_failures += 1;
            
            // Update status based on failure count
            chain_health.status = match chain_health.consecutive_failures {
                1..=2 => ChainHealthStatus::Degraded,
                3..=5 => ChainHealthStatus::Unhealthy,
                _ => ChainHealthStatus::Offline,
            };
        }
        Ok(())
    }

    fn create_coordination_result(&self, operation: &CrossChainOperation, start_time: Instant) -> CoordinationResult {
        CoordinationResult {
            operation_id: operation.operation_id.clone(),
            success: matches!(operation.status, OperationStatus::Success),
            successful_chains: operation.completed_chains.clone(),
            failed_chains: operation.failed_chains.clone(),
            total_time_ms: start_time.elapsed().as_millis() as u64,
            retry_attempts: operation.retry_count,
        }
    }
}

#[derive(Debug, Clone)]
struct BatchResult {
    successful_chains: Vec<u64>,
    failed_chains: Vec<u64>,
}

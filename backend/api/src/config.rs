use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub blockchain: BlockchainConfig,
    pub oracle: OracleConfig,
    pub compliance: ComplianceConfig,
    pub treasury: TreasuryConfig,
    pub zk_nav: ZKNavConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitingConfig,
    pub metrics: MetricsConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
    pub enable_cors: bool,
    pub enable_compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub enable_logging: bool,
    pub migration_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
    pub enable_cluster: bool,
    pub key_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub solana: SolanaConfig,
    pub ethereum: EthereumConfig,
    pub bitcoin: BitcoinConfig,
    pub starknet: StarknetConfig,
    pub icp: ICPConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment: String,
    pub vault_program_id: String,
    pub governance_program_id: String,
    pub keypair_path: String,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub chain_id: u64,
    pub governance_contract: String,
    pub ccip_router: String,
    pub private_key: String,
    pub gas_limit: u64,
    pub gas_price_gwei: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinConfig {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
    pub network: String, // mainnet, testnet, regtest
    pub babylon_endpoint: String,
    pub finality_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarknetConfig {
    pub rpc_url: String,
    pub account_address: String,
    pub private_key: String,
    pub zk_nav_contract: String,
    pub max_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ICPConfig {
    pub replica_url: String,
    pub canister_id: String,
    pub identity_pem_path: String,
    pub agent_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConfig {
    pub chainlink: ChainlinkConfig,
    pub switchboard: SwitchboardConfig,
    pub pyth: PythConfig,
    pub update_interval_seconds: u64,
    pub price_deviation_threshold: f64,
    pub confidence_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainlinkConfig {
    pub ccip_router: String,
    pub price_feeds: HashMap<String, String>,
    pub functions_router: String,
    pub subscription_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchboardConfig {
    pub program_id: String,
    pub aggregator_accounts: HashMap<String, String>,
    pub queue_account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythConfig {
    pub program_id: String,
    pub price_accounts: HashMap<String, String>,
    pub confidence_interval: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub kyc_providers: Vec<KYCProviderConfig>,
    pub jurisdictions: Vec<JurisdictionConfig>,
    pub risk_scoring: RiskScoringConfig,
    pub aml_screening: AMLScreeningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KYCProviderConfig {
    pub name: String,
    pub api_url: String,
    pub api_key: String,
    pub verification_levels: Vec<String>,
    pub supported_documents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionConfig {
    pub country_code: String,
    pub allowed: bool,
    pub min_investment: u64,
    pub max_investment: Option<u64>,
    pub accredited_only: bool,
    pub additional_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScoringConfig {
    pub model_endpoint: String,
    pub api_key: String,
    pub score_threshold: u8,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AMLScreeningConfig {
    pub provider: String,
    pub api_url: String,
    pub api_key: String,
    pub screening_lists: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryConfig {
    pub rebalancing_interval_hours: u64,
    pub risk_limits: RiskLimitsConfig,
    pub asset_allocations: HashMap<String, AllocationConfig>,
    pub emergency_thresholds: EmergencyThresholdsConfig,
    pub ai_assistant: AIAssistantConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitsConfig {
    pub max_var_95_percent: f64,
    pub max_volatility_percent: f64,
    pub max_drawdown_percent: f64,
    pub min_sharpe_ratio: f64,
    pub max_concentration_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationConfig {
    pub min_percent: f64,
    pub max_percent: f64,
    pub target_percent: f64,
    pub rebalance_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyThresholdsConfig {
    pub tvl_drop_percent: f64,
    pub nav_drift_percent: f64,
    pub liquidity_ratio_min: f64,
    pub redemption_queue_max_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAssistantConfig {
    pub provider: String, // "openai", "anthropic", "local"
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_treasury_analysis: bool,
    pub enable_risk_assessment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKNavConfig {
    pub starknet_contract: String,
    pub computation_interval_minutes: u64,
    pub proof_verification_timeout_seconds: u64,
    pub max_drift_threshold_percent: f64,
    pub aggregation_enabled: bool,
    pub recursive_proof_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub refresh_token_expiry_days: u64,
    pub password_hash_cost: u32,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u64,
    pub require_2fa: bool,
    pub session_timeout_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enable_per_user_limits: bool,
    pub premium_user_multiplier: u32,
    pub admin_exemption: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
    pub enable_jaeger: bool,
    pub jaeger_endpoint: String,
    pub enable_custom_metrics: bool,
    pub metrics_retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_https: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub enable_hsts: bool,
    pub enable_csp: bool,
    pub allowed_origins: Vec<String>,
    pub api_key_header: String,
    pub enable_request_signing: bool,
    pub post_quantum_signatures: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    pub fn load_from_env() -> Result<Self> {
        // Load configuration from environment variables
        // This would be implemented to read from env vars with fallback to defaults
        todo!("Implement environment variable configuration loading")
    }

    pub fn validate(&self) -> Result<()> {
        // Validate configuration values
        if self.server.port == 0 {
            return Err(anyhow::anyhow!("Server port cannot be 0"));
        }

        if self.database.url.is_empty() {
            return Err(anyhow::anyhow!("Database URL cannot be empty"));
        }

        if self.auth.jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!("JWT secret must be at least 32 characters"));
        }

        // Validate blockchain configurations
        if self.blockchain.solana.rpc_url.is_empty() {
            return Err(anyhow::anyhow!("Solana RPC URL cannot be empty"));
        }

        if self.blockchain.ethereum.rpc_url.is_empty() {
            return Err(anyhow::anyhow!("Ethereum RPC URL cannot be empty"));
        }

        // Validate treasury risk limits
        if self.treasury.risk_limits.max_var_95_percent <= 0.0 {
            return Err(anyhow::anyhow!("Max VaR must be positive"));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 2102,
                workers: num_cpus::get(),
                max_connections: 1000,
                request_timeout_seconds: 30,
                enable_cors: true,
                enable_compression: true,
            },
            database: DatabaseConfig {
                url: "postgresql://rtf:rtf@localhost/rtf".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 600,
                max_lifetime_seconds: 3600,
                enable_logging: false,
                migration_path: "migrations".to_string(),
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 20,
                connection_timeout_seconds: 5,
                command_timeout_seconds: 5,
                enable_cluster: false,
                key_prefix: "rtf:".to_string(),
            },
            blockchain: BlockchainConfig {
                solana: SolanaConfig {
                    rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                    ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
                    commitment: "confirmed".to_string(),
                    vault_program_id: "RTFVau1tAdvancedSPLTokenVau1tProgram11111111".to_string(),
                    governance_program_id: "RTFGovAdvancedDAOGovernanceProgram1111111".to_string(),
                    keypair_path: "keypairs/solana.json".to_string(),
                    max_retries: 3,
                    timeout_seconds: 30,
                },
                ethereum: EthereumConfig {
                    rpc_url: "https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
                    ws_url: "wss://eth-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
                    chain_id: 1,
                    governance_contract: "0x0000000000000000000000000000000000000000".to_string(),
                    ccip_router: "0x0000000000000000000000000000000000000000".to_string(),
                    private_key: "".to_string(),
                    gas_limit: 500000,
                    gas_price_gwei: 20,
                },
                bitcoin: BitcoinConfig {
                    rpc_url: "https://bitcoin-mainnet.example.com".to_string(),
                    rpc_user: "user".to_string(),
                    rpc_password: "password".to_string(),
                    network: "mainnet".to_string(),
                    babylon_endpoint: "https://babylon-api.example.com".to_string(),
                    finality_provider: "".to_string(),
                },
                starknet: StarknetConfig {
                    rpc_url: "https://starknet-mainnet.g.alchemy.com/v2/YOUR_KEY".to_string(),
                    account_address: "0x0000000000000000000000000000000000000000".to_string(),
                    private_key: "".to_string(),
                    zk_nav_contract: "0x0000000000000000000000000000000000000000".to_string(),
                    max_fee: "1000000000000000".to_string(),
                },
                icp: ICPConfig {
                    replica_url: "https://ic0.app".to_string(),
                    canister_id: "rdmx6-jaaaa-aaaah-qcaiq-cai".to_string(),
                    identity_pem_path: "identity.pem".to_string(),
                    agent_timeout_seconds: 30,
                },
            },
            oracle: OracleConfig {
                chainlink: ChainlinkConfig {
                    ccip_router: "0x0000000000000000000000000000000000000000".to_string(),
                    price_feeds: HashMap::new(),
                    functions_router: "0x0000000000000000000000000000000000000000".to_string(),
                    subscription_id: 0,
                },
                switchboard: SwitchboardConfig {
                    program_id: "SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f".to_string(),
                    aggregator_accounts: HashMap::new(),
                    queue_account: "".to_string(),
                },
                pyth: PythConfig {
                    program_id: "FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH".to_string(),
                    price_accounts: HashMap::new(),
                    confidence_interval: 0.95,
                },
                update_interval_seconds: 60,
                price_deviation_threshold: 0.05,
                confidence_threshold: 80,
            },
            compliance: ComplianceConfig {
                kyc_providers: vec![],
                jurisdictions: vec![],
                risk_scoring: RiskScoringConfig {
                    model_endpoint: "".to_string(),
                    api_key: "".to_string(),
                    score_threshold: 70,
                    factors: vec![],
                },
                aml_screening: AMLScreeningConfig {
                    provider: "".to_string(),
                    api_url: "".to_string(),
                    api_key: "".to_string(),
                    screening_lists: vec![],
                },
            },
            treasury: TreasuryConfig {
                rebalancing_interval_hours: 24,
                risk_limits: RiskLimitsConfig {
                    max_var_95_percent: 5.0,
                    max_volatility_percent: 20.0,
                    max_drawdown_percent: 10.0,
                    min_sharpe_ratio: 0.5,
                    max_concentration_percent: 25.0,
                },
                asset_allocations: HashMap::new(),
                emergency_thresholds: EmergencyThresholdsConfig {
                    tvl_drop_percent: 25.0,
                    nav_drift_percent: 10.0,
                    liquidity_ratio_min: 0.1,
                    redemption_queue_max_percent: 50.0,
                },
                ai_assistant: AIAssistantConfig {
                    provider: "openai".to_string(),
                    api_key: "".to_string(),
                    model: "gpt-4".to_string(),
                    max_tokens: 2048,
                    temperature: 0.1,
                    enable_treasury_analysis: true,
                    enable_risk_assessment: true,
                },
            },
            zk_nav: ZKNavConfig {
                starknet_contract: "0x0000000000000000000000000000000000000000".to_string(),
                computation_interval_minutes: 60,
                proof_verification_timeout_seconds: 300,
                max_drift_threshold_percent: 5.0,
                aggregation_enabled: true,
                recursive_proof_depth: 3,
            },
            auth: AuthConfig {
                jwt_secret: "your-super-secret-jwt-key-change-this-in-production".to_string(),
                jwt_expiry_hours: 24,
                refresh_token_expiry_days: 30,
                password_hash_cost: 12,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
                require_2fa: false,
                session_timeout_minutes: 60,
            },
            rate_limiting: RateLimitingConfig {
                requests_per_minute: 100,
                burst_size: 20,
                enable_per_user_limits: true,
                premium_user_multiplier: 5,
                admin_exemption: true,
            },
            metrics: MetricsConfig {
                enable_prometheus: true,
                prometheus_port: 9090,
                enable_jaeger: false,
                jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
                enable_custom_metrics: true,
                metrics_retention_days: 30,
            },
            security: SecurityConfig {
                enable_https: false,
                tls_cert_path: "certs/cert.pem".to_string(),
                tls_key_path: "certs/key.pem".to_string(),
                enable_hsts: true,
                enable_csp: true,
                allowed_origins: vec!["http://localhost:2101".to_string()],
                api_key_header: "X-API-Key".to_string(),
                enable_request_signing: false,
                post_quantum_signatures: false,
            },
        }
    }
}

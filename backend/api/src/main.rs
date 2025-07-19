use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod middleware_custom;
mod models;
mod services;
mod utils;
mod blockchain;
mod compliance;
mod oracle;
mod treasury;
mod zk_nav;

use config::Config;
use handlers::*;
use middleware_custom::*;
use services::*;

/// RTF API Server - Advanced Multi-Chain Fund Management
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    /// Server port
    #[arg(short, long, default_value = "2102")]
    port: u16,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Enable development mode
    #[arg(long)]
    dev: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub database: Arc<DatabaseService>,
    pub redis: Arc<RedisService>,
    pub blockchain: Arc<BlockchainService>,
    pub oracle: Arc<OracleService>,
    pub compliance: Arc<ComplianceService>,
    pub treasury: Arc<TreasuryService>,
    pub zk_nav: Arc<ZKNavService>,
    pub cross_chain: Arc<CrossChainService>,
    pub llm_agent: Arc<LlmAgentService>,
    pub exposure_detector: Arc<ExposureDetectorService>,
    pub emergency_handler: Arc<EmergencyHandlerService>,
    pub post_quantum: Arc<PostQuantumService>,
    pub auth: Arc<AuthService>,
    pub rate_limiter: Arc<RateLimiterService>,
    pub metrics: Arc<MetricsService>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    init_tracing(&args.log_level)?;
    
    info!("🚀 Starting RTF API Server v{}", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config = Arc::new(Config::load(&args.config)?);
    info!("📋 Configuration loaded from {}", args.config);
    
    // Initialize services
    let app_state = initialize_services(config.clone()).await?;
    info!("🔧 All services initialized successfully");
    
    // Build application router
    let app = build_router(app_state.clone()).await?;
    info!("🌐 Router configured with all endpoints");
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🎯 Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("👋 RTF API Server shutdown complete");
    Ok(())
}

async fn initialize_services(config: Arc<Config>) -> Result<AppState> {
    info!("🔄 Initializing core services...");
    
    // Database service
    let database = Arc::new(DatabaseService::new(&config.database).await?);
    info!("✅ Database service initialized");
    
    // Redis service
    let redis = Arc::new(RedisService::new(&config.redis).await?);
    info!("✅ Redis service initialized");
    
    // Blockchain service (multi-chain)
    let blockchain = Arc::new(BlockchainService::new(&config.blockchain).await?);
    info!("✅ Blockchain service initialized (Solana, Ethereum, Bitcoin)");
    
    // Oracle service
    let oracle = Arc::new(OracleService::new(&config.oracle, blockchain.clone()).await?);
    info!("✅ Oracle service initialized (Chainlink, Switchboard)");
    
    // Compliance service
    let compliance = Arc::new(ComplianceService::new(&config.compliance).await?);
    info!("✅ Compliance service initialized (KYC, AML, Jurisdictional)");
    
    // Treasury service
    let treasury = Arc::new(TreasuryService::new(
        &config.treasury,
        database.clone(),
        blockchain.clone(),
        oracle.clone(),
    ).await?);
    info!("✅ Treasury service initialized");
    
    // zkNAV service with advanced features
    let zk_nav = Arc::new(ZKNavService::new_with_recursive_proofs(&config.zk_nav, blockchain.clone()).await?);
    info!("✅ zkNAV service initialized (Starknet integration, recursive proofs, drift enforcement)");

    // Cross-chain service with CCIP integration
    let cross_chain = Arc::new(CrossChainService::new_with_ccip(&config.cross_chain, blockchain.clone()).await?);
    info!("✅ Cross-chain service initialized (Chainlink CCIP, Babylon, ICP Chain Fusion)");

    // LLM Agent service with integrity verification
    let llm_agent = Arc::new(LlmAgentService::new_with_integrity(&config.llm_agent).await?);
    info!("✅ LLM Agent service initialized (semantic integrity, governance simulation)");

    // Exposure detector service with graph analysis
    let exposure_detector = Arc::new(ExposureDetectorService::new_with_graph(&config.exposure_detector).await?);
    info!("✅ Exposure detector service initialized (circular dependency detection, fund isolation)");

    // Emergency handler service with circuit breaker
    let emergency_handler = Arc::new(EmergencyHandlerService::new_with_circuit_breaker(&config.emergency).await?);
    info!("✅ Emergency handler service initialized (circuit breaker, DAO emergency session lock)");

    // Post-quantum cryptography service
    let post_quantum = Arc::new(PostQuantumService::new_with_dilithium(&config.post_quantum).await?);
    info!("✅ Post-quantum service initialized (Dilithium512, dual-signature)");

    // Authentication service with enhanced security
    let auth = Arc::new(AuthService::new_with_mfa(&config.auth, database.clone(), post_quantum.clone()).await?);
    info!("✅ Authentication service initialized (MFA, post-quantum security)");
    
    // Rate limiter service
    let rate_limiter = Arc::new(RateLimiterService::new(&config.rate_limiting)?);
    info!("✅ Rate limiter service initialized");
    
    // Metrics service
    let metrics = Arc::new(MetricsService::new(&config.metrics)?);
    info!("✅ Metrics service initialized");
    
    Ok(AppState {
        config,
        database,
        redis,
        blockchain,
        oracle,
        compliance,
        treasury,
        zk_nav,
        cross_chain,
        llm_agent,
        exposure_detector,
        emergency_handler,
        post_quantum,
        auth,
        rate_limiter,
        metrics,
    })
}

async fn build_router(state: AppState) -> Result<Router> {
    let api_v1 = Router::new()
        // Health and status endpoints
        .route("/health", get(health_check))
        .route("/status", get(system_status))
        .route("/metrics", get(prometheus_metrics))
        
        // Authentication endpoints
        .route("/auth/login", post(auth_login))
        .route("/auth/refresh", post(auth_refresh))
        .route("/auth/logout", post(auth_logout))
        
        // Vault management endpoints with advanced features
        .route("/vaults", get(list_vaults).post(create_vault))
        .route("/vaults/:vault_id", get(get_vault).put(update_vault).delete(delete_vault))
        .route("/vaults/:vault_id/deposit", post(deposit_to_vault_with_compliance))
        .route("/vaults/:vault_id/redeem", post(request_redemption_advanced))
        .route("/vaults/:vault_id/nav", get(get_vault_nav).post(update_vault_nav_with_zk))
        .route("/vaults/:vault_id/emergency", post(emergency_vault_action))
        .route("/vaults/:vault_id/rebalance", post(rebalance_vault_tranches))
        
        // Tranche management
        .route("/vaults/:vault_id/tranches", get(list_tranches).post(create_tranche))
        .route("/vaults/:vault_id/tranches/:tranche_id", get(get_tranche).put(update_tranche))
        
        // Portfolio and holdings
        .route("/vaults/:vault_id/holdings", get(get_holdings))
        .route("/vaults/:vault_id/performance", get(get_performance_metrics))
        .route("/vaults/:vault_id/risk", get(get_risk_metrics))
        
        // Governance endpoints
        .route("/governance/proposals", get(list_proposals).post(create_proposal))
        .route("/governance/proposals/:proposal_id", get(get_proposal))
        .route("/governance/proposals/:proposal_id/vote", post(vote_on_proposal))
        .route("/governance/daos", get(list_daos))
        
        // Compliance endpoints
        .route("/compliance/kyc", post(submit_kyc))
        .route("/compliance/kyc/:user_id", get(get_kyc_status))
        .route("/compliance/jurisdictions", get(list_jurisdictions))
        .route("/compliance/verify", post(verify_compliance))
        
        // Oracle and pricing
        .route("/oracles/prices", get(get_current_prices))
        .route("/oracles/feeds", get(list_oracle_feeds))
        .route("/oracles/nav", post(submit_nav_data))
        
        // Treasury management
        .route("/treasury/reserves", get(get_treasury_reserves))
        .route("/treasury/rebalance", post(rebalance_treasury))
        .route("/treasury/analysis", get(get_treasury_analysis))
        
        // Cross-chain operations
        .route("/cross-chain/bridges", get(list_bridges))
        .route("/cross-chain/transfer", post(initiate_cross_chain_transfer))
        .route("/cross-chain/status/:tx_id", get(get_cross_chain_status))
        
        // zkNAV endpoints
        .route("/zk-nav/compute", post(compute_zk_nav))
        .route("/zk-nav/verify", post(verify_nav_proof))
        .route("/zk-nav/history/:fund_id", get(get_nav_history))
        
        // Analytics and reporting
        .route("/analytics/dashboard", get(get_dashboard_data))
        .route("/analytics/reports", get(list_reports).post(generate_report))
        .route("/analytics/exposure", get(analyze_fund_exposure))
        
        // Admin endpoints (restricted)
        .route("/admin/users", get(list_users).post(create_user))
        .route("/admin/users/:user_id", get(get_user).put(update_user).delete(delete_user))
        .route("/admin/system", get(get_system_info))
        .route("/admin/emergency", post(emergency_pause));

    let app = Router::new()
        .nest("/api/v1", api_v1)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    rate_limiting_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    metrics_middleware,
                ))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    security_headers_middleware,
                )),
        )
        .with_state(state);

    Ok(app)
}

fn init_tracing(log_level: &str) -> Result<()> {
    let level = match log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("rtf_api={},tower_http=debug", level).into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }

    info!("Starting graceful shutdown...");
}

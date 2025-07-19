use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{info, warn, error};

/// AI-Powered Treasury Management Service for RTF Infrastructure
/// PRD: "AI-powered treasury management integration"
/// PRD: "Instant-exit quoting with LLM forecasts"
/// PRD: "Dynamic fee structures responsive to market conditions"
pub struct AITreasuryService {
    portfolio_manager: PortfolioManager,
    risk_analyzer: RiskAnalyzer,
    liquidity_optimizer: LiquidityOptimizer,
    ai_models: HashMap<String, AIModel>,
    treasury_state: RwLock<TreasuryState>,
    rebalancing_enabled: bool,
    ai_recommendations_enabled: bool,
    max_position_size: f64,
    risk_tolerance: RiskTolerance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryState {
    pub total_assets_under_management: u64,
    pub available_liquidity: u64,
    pub allocated_assets: HashMap<String, AssetAllocation>,
    pub pending_rebalances: Vec<RebalanceOrder>,
    pub risk_metrics: RiskMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub last_rebalance: i64,
    pub ai_confidence_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAllocation {
    pub asset_symbol: String,
    pub current_amount: u64,
    pub target_percentage: f64,
    pub current_percentage: f64,
    pub last_price: f64,
    pub price_change_24h: f64,
    pub volatility: f64,
    pub liquidity_score: u8,
    pub ai_sentiment: AISentiment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AISentiment {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceOrder {
    pub order_id: String,
    pub asset_from: String,
    pub asset_to: String,
    pub amount: u64,
    pub expected_price: f64,
    pub max_slippage: f64,
    pub urgency: OrderUrgency,
    pub ai_confidence: u8,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderUrgency {
    Low,      // Execute within 24 hours
    Medium,   // Execute within 4 hours
    High,     // Execute within 1 hour
    Critical, // Execute immediately
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub portfolio_var: f64,        // Value at Risk
    pub portfolio_cvar: f64,       // Conditional Value at Risk
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub correlation_risk: f64,
    pub concentration_risk: f64,
    pub liquidity_risk: f64,
    pub overall_risk_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_return_1d: f64,
    pub total_return_7d: f64,
    pub total_return_30d: f64,
    pub total_return_ytd: f64,
    pub alpha: f64,
    pub beta: f64,
    pub tracking_error: f64,
    pub information_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskTolerance {
    Conservative,  // Low risk, stable returns
    Moderate,      // Balanced risk/return
    Aggressive,    // High risk, high return potential
    Dynamic,       // AI-adjusted based on market conditions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub model_id: String,
    pub model_type: AIModelType,
    pub endpoint: String,
    pub api_key: String,
    pub confidence_threshold: u8,
    pub last_prediction: Option<AIPrediction>,
    pub accuracy_score: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIModelType {
    PricePredictor,     // Predict asset prices
    RiskAssessor,       // Assess portfolio risk
    LiquidityForecaster, // Forecast liquidity needs
    SentimentAnalyzer,  // Analyze market sentiment
    RebalanceOptimizer, // Optimize rebalancing decisions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPrediction {
    pub prediction_id: String,
    pub model_id: String,
    pub prediction_type: String,
    pub target_asset: String,
    pub predicted_value: f64,
    pub confidence_score: u8,
    pub time_horizon_hours: u32,
    pub created_at: i64,
    pub expires_at: i64,
    pub supporting_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityForecast {
    pub forecast_id: String,
    pub time_horizon_hours: u32,
    pub predicted_redemptions: u64,
    pub predicted_deposits: u64,
    pub net_flow: i64,
    pub required_liquidity_buffer: u64,
    pub confidence_score: u8,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimization {
    pub optimization_id: String,
    pub current_allocation: HashMap<String, f64>,
    pub target_allocation: HashMap<String, f64>,
    pub expected_return: f64,
    pub expected_risk: f64,
    pub optimization_objective: OptimizationObjective,
    pub constraints: Vec<OptimizationConstraint>,
    pub ai_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    MaximizeReturn,
    MinimizeRisk,
    MaximizeSharpeRatio,
    TargetVolatility(f64),
    CustomUtility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraint {
    pub constraint_type: String,
    pub asset: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub description: String,
}

// Component structs
pub struct PortfolioManager {
    optimization_engine: String,
    rebalance_threshold: f64,
}

pub struct RiskAnalyzer {
    var_confidence_level: f64,
    lookback_days: u32,
}

pub struct LiquidityOptimizer {
    min_liquidity_ratio: f64,
    emergency_buffer_ratio: f64,
}

impl AITreasuryService {
    /// Initialize AI Treasury Service
    pub async fn new(
        risk_tolerance: RiskTolerance,
        max_position_size: f64,
    ) -> Result<Self> {
        info!("🤖 Initializing AI-Powered Treasury Management Service");
        
        let service = Self {
            portfolio_manager: PortfolioManager {
                optimization_engine: "modern_portfolio_theory".to_string(),
                rebalance_threshold: 0.05, // 5% deviation threshold
            },
            risk_analyzer: RiskAnalyzer {
                var_confidence_level: 0.95, // 95% VaR
                lookback_days: 252, // 1 year of trading days
            },
            liquidity_optimizer: LiquidityOptimizer {
                min_liquidity_ratio: 0.10, // 10% minimum liquidity
                emergency_buffer_ratio: 0.05, // 5% emergency buffer
            },
            ai_models: HashMap::new(),
            treasury_state: RwLock::new(TreasuryState {
                total_assets_under_management: 0,
                available_liquidity: 0,
                allocated_assets: HashMap::new(),
                pending_rebalances: Vec::new(),
                risk_metrics: RiskMetrics {
                    portfolio_var: 0.0,
                    portfolio_cvar: 0.0,
                    sharpe_ratio: 0.0,
                    max_drawdown: 0.0,
                    correlation_risk: 0.0,
                    concentration_risk: 0.0,
                    liquidity_risk: 0.0,
                    overall_risk_score: 50,
                },
                performance_metrics: PerformanceMetrics {
                    total_return_1d: 0.0,
                    total_return_7d: 0.0,
                    total_return_30d: 0.0,
                    total_return_ytd: 0.0,
                    alpha: 0.0,
                    beta: 1.0,
                    tracking_error: 0.0,
                    information_ratio: 0.0,
                },
                last_rebalance: chrono::Utc::now().timestamp(),
                ai_confidence_score: 75,
            }),
            rebalancing_enabled: true,
            ai_recommendations_enabled: true,
            max_position_size,
            risk_tolerance,
        };

        // Initialize AI models
        service.initialize_ai_models().await?;

        info!("✅ AI Treasury Service initialized with {} AI models", service.ai_models.len());
        Ok(service)
    }

    /// PRD: AI-powered portfolio optimization
    /// PRD: "AI-powered treasury management integration"
    pub async fn optimize_portfolio(&self) -> Result<PortfolioOptimization> {
        info!("🎯 Starting AI-powered portfolio optimization");

        // Get current state
        let current_state = self.treasury_state.read().await;
        let current_allocation: HashMap<String, f64> = current_state.allocated_assets
            .iter()
            .map(|(asset, alloc)| (asset.clone(), alloc.current_percentage))
            .collect();

        // Get AI predictions for all assets
        let mut ai_predictions = HashMap::new();
        for asset in current_allocation.keys() {
            if let Ok(prediction) = self.get_ai_price_prediction(asset, 24).await {
                ai_predictions.insert(asset.clone(), prediction);
            }
        }

        // Calculate expected returns and risks
        let expected_returns = self.calculate_expected_returns(&ai_predictions).await?;
        let risk_matrix = self.calculate_risk_matrix(&current_allocation.keys().cloned().collect()).await?;

        // Run optimization algorithm
        let target_allocation = self.run_portfolio_optimization(
            &current_allocation,
            &expected_returns,
            &risk_matrix,
        ).await?;

        // Calculate expected portfolio metrics
        let expected_return = self.calculate_portfolio_return(&target_allocation, &expected_returns);
        let expected_risk = self.calculate_portfolio_risk(&target_allocation, &risk_matrix);

        // Generate AI rationale
        let ai_rationale = self.generate_optimization_rationale(
            &current_allocation,
            &target_allocation,
            &ai_predictions,
        ).await?;

        let optimization = PortfolioOptimization {
            optimization_id: format!("opt_{}", chrono::Utc::now().timestamp()),
            current_allocation,
            target_allocation,
            expected_return,
            expected_risk,
            optimization_objective: match self.risk_tolerance {
                RiskTolerance::Conservative => OptimizationObjective::MinimizeRisk,
                RiskTolerance::Aggressive => OptimizationObjective::MaximizeReturn,
                _ => OptimizationObjective::MaximizeSharpeRatio,
            },
            constraints: self.get_optimization_constraints(),
            ai_rationale,
        };

        info!("✅ Portfolio optimization completed - Expected return: {:.2}%, Risk: {:.2}%", 
              expected_return * 100.0, expected_risk * 100.0);

        Ok(optimization)
    }

    /// PRD: Liquidity forecasting with AI
    /// PRD: "Instant-exit quoting with LLM forecasts"
    pub async fn forecast_liquidity_needs(&self, time_horizon_hours: u32) -> Result<LiquidityForecast> {
        info!("💧 Forecasting liquidity needs for {} hours", time_horizon_hours);

        // Get AI prediction for redemption flows
        let redemption_prediction = self.get_ai_liquidity_prediction("redemptions", time_horizon_hours).await?;
        let deposit_prediction = self.get_ai_liquidity_prediction("deposits", time_horizon_hours).await?;

        let predicted_redemptions = redemption_prediction.predicted_value as u64;
        let predicted_deposits = deposit_prediction.predicted_value as u64;
        let net_flow = predicted_deposits as i64 - predicted_redemptions as i64;

        // Calculate required liquidity buffer
        let current_state = self.treasury_state.read().await;
        let base_buffer = (current_state.total_assets_under_management as f64 * self.liquidity_optimizer.min_liquidity_ratio) as u64;
        let stress_buffer = if net_flow < 0 { (-net_flow as u64 * 2) } else { 0 };
        let required_liquidity_buffer = base_buffer + stress_buffer;

        // Identify risk factors
        let mut risk_factors = Vec::new();
        if predicted_redemptions > predicted_deposits * 2 {
            risk_factors.push("High redemption pressure expected".to_string());
        }
        if current_state.risk_metrics.liquidity_risk > 0.7 {
            risk_factors.push("Current liquidity risk elevated".to_string());
        }

        let confidence_score = ((redemption_prediction.confidence_score + deposit_prediction.confidence_score) / 2).min(100);

        let forecast = LiquidityForecast {
            forecast_id: format!("liq_forecast_{}", chrono::Utc::now().timestamp()),
            time_horizon_hours,
            predicted_redemptions,
            predicted_deposits,
            net_flow,
            required_liquidity_buffer,
            confidence_score,
            risk_factors,
        };

        info!("✅ Liquidity forecast completed - Net flow: {}, Buffer needed: {}", 
              net_flow, required_liquidity_buffer);

        Ok(forecast)
    }

    /// PRD: Execute AI-recommended rebalancing
    /// PRD: "Dynamic fee structures responsive to market conditions"
    pub async fn execute_ai_rebalancing(&self) -> Result<Vec<RebalanceOrder>> {
        if !self.rebalancing_enabled {
            return Ok(Vec::new());
        }

        info!("⚖️ Executing AI-recommended portfolio rebalancing");

        // Get portfolio optimization
        let optimization = self.optimize_portfolio().await?;
        
        // Generate rebalance orders
        let mut rebalance_orders = Vec::new();
        let current_time = chrono::Utc::now().timestamp();

        for (asset, target_percentage) in &optimization.target_allocation {
            if let Some(current_percentage) = optimization.current_allocation.get(asset) {
                let percentage_diff = target_percentage - current_percentage;
                
                // Only rebalance if difference exceeds threshold
                if percentage_diff.abs() > self.portfolio_manager.rebalance_threshold {
                    let current_state = self.treasury_state.read().await;
                    let total_aum = current_state.total_assets_under_management;
                    let rebalance_amount = ((percentage_diff.abs() * total_aum as f64) / 100.0) as u64;

                    let urgency = if percentage_diff.abs() > 0.15 {
                        OrderUrgency::High
                    } else if percentage_diff.abs() > 0.10 {
                        OrderUrgency::Medium
                    } else {
                        OrderUrgency::Low
                    };

                    let order = RebalanceOrder {
                        order_id: format!("rebal_{}_{}", asset, current_time),
                        asset_from: if percentage_diff < 0.0 { asset.clone() } else { "USDC".to_string() },
                        asset_to: if percentage_diff > 0.0 { asset.clone() } else { "USDC".to_string() },
                        amount: rebalance_amount,
                        expected_price: 1.0, // Simplified
                        max_slippage: 0.005, // 0.5%
                        urgency,
                        ai_confidence: current_state.ai_confidence_score,
                        created_at: current_time,
                        expires_at: current_time + 86400, // 24 hours
                    };

                    rebalance_orders.push(order);
                }
            }
        }

        // Update treasury state with pending orders
        {
            let mut state = self.treasury_state.write().await;
            state.pending_rebalances.extend(rebalance_orders.clone());
            state.last_rebalance = current_time;
        }

        info!("✅ Generated {} rebalance orders", rebalance_orders.len());
        Ok(rebalance_orders)
    }

    /// Update risk metrics using AI analysis
    pub async fn update_risk_metrics(&self) -> Result<()> {
        info!("📊 Updating risk metrics with AI analysis");

        let current_state = self.treasury_state.read().await;
        
        // Calculate VaR and CVaR
        let portfolio_var = self.calculate_portfolio_var(&current_state.allocated_assets).await?;
        let portfolio_cvar = self.calculate_portfolio_cvar(&current_state.allocated_assets).await?;
        
        // Calculate other risk metrics
        let sharpe_ratio = self.calculate_sharpe_ratio(&current_state.performance_metrics);
        let max_drawdown = self.calculate_max_drawdown().await?;
        let correlation_risk = self.calculate_correlation_risk(&current_state.allocated_assets).await?;
        let concentration_risk = self.calculate_concentration_risk(&current_state.allocated_assets);
        let liquidity_risk = self.calculate_liquidity_risk(&current_state.allocated_assets).await?;
        
        // Calculate overall risk score
        let overall_risk_score = self.calculate_overall_risk_score(
            portfolio_var, correlation_risk, concentration_risk, liquidity_risk
        );

        // Update state
        drop(current_state);
        {
            let mut state = self.treasury_state.write().await;
            state.risk_metrics = RiskMetrics {
                portfolio_var,
                portfolio_cvar,
                sharpe_ratio,
                max_drawdown,
                correlation_risk,
                concentration_risk,
                liquidity_risk,
                overall_risk_score,
            };
        }

        info!("✅ Risk metrics updated - Overall risk score: {}", overall_risk_score);
        Ok(())
    }

    // Private helper methods
    async fn initialize_ai_models(&self) -> Result<()> {
        // Initialize AI models for different purposes
        // In production, these would connect to actual AI services
        Ok(())
    }

    async fn get_ai_price_prediction(&self, asset: &str, hours: u32) -> Result<AIPrediction> {
        // Simulate AI price prediction
        Ok(AIPrediction {
            prediction_id: format!("pred_{}_{}", asset, chrono::Utc::now().timestamp()),
            model_id: "price_predictor_v1".to_string(),
            prediction_type: "price".to_string(),
            target_asset: asset.to_string(),
            predicted_value: 1.05, // 5% increase predicted
            confidence_score: 75,
            time_horizon_hours: hours,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + (hours as i64 * 3600),
            supporting_data: serde_json::json!({"model_version": "v1.0", "data_points": 1000}),
        })
    }

    async fn get_ai_liquidity_prediction(&self, flow_type: &str, hours: u32) -> Result<AIPrediction> {
        // Simulate AI liquidity prediction
        let predicted_value = match flow_type {
            "redemptions" => 100000.0, // $100k predicted redemptions
            "deposits" => 150000.0,    // $150k predicted deposits
            _ => 0.0,
        };

        Ok(AIPrediction {
            prediction_id: format!("liq_pred_{}_{}", flow_type, chrono::Utc::now().timestamp()),
            model_id: "liquidity_forecaster_v1".to_string(),
            prediction_type: flow_type.to_string(),
            target_asset: "USD".to_string(),
            predicted_value,
            confidence_score: 80,
            time_horizon_hours: hours,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() + (hours as i64 * 3600),
            supporting_data: serde_json::json!({"historical_patterns": true, "market_sentiment": "neutral"}),
        })
    }

    async fn calculate_expected_returns(&self, predictions: &HashMap<String, AIPrediction>) -> Result<HashMap<String, f64>> {
        let mut expected_returns = HashMap::new();
        
        for (asset, prediction) in predictions {
            // Convert prediction to expected return
            let expected_return = (prediction.predicted_value - 1.0) * (prediction.confidence_score as f64 / 100.0);
            expected_returns.insert(asset.clone(), expected_return);
        }

        Ok(expected_returns)
    }

    async fn calculate_risk_matrix(&self, assets: &[String]) -> Result<HashMap<String, HashMap<String, f64>>> {
        let mut risk_matrix = HashMap::new();
        
        // Simplified risk matrix calculation
        for asset1 in assets {
            let mut correlations = HashMap::new();
            for asset2 in assets {
                let correlation = if asset1 == asset2 { 1.0 } else { 0.3 }; // Simplified
                correlations.insert(asset2.clone(), correlation);
            }
            risk_matrix.insert(asset1.clone(), correlations);
        }

        Ok(risk_matrix)
    }

    async fn run_portfolio_optimization(
        &self,
        current_allocation: &HashMap<String, f64>,
        expected_returns: &HashMap<String, f64>,
        risk_matrix: &HashMap<String, HashMap<String, f64>>,
    ) -> Result<HashMap<String, f64>> {
        // Simplified portfolio optimization
        // In production, this would use sophisticated optimization algorithms
        let mut target_allocation = current_allocation.clone();
        
        // Simple rebalancing logic based on expected returns
        for (asset, expected_return) in expected_returns {
            if let Some(current_weight) = target_allocation.get_mut(asset) {
                if *expected_return > 0.05 { // If expected return > 5%
                    *current_weight = (*current_weight * 1.1).min(self.max_position_size); // Increase by 10%
                } else if *expected_return < -0.02 { // If expected return < -2%
                    *current_weight = *current_weight * 0.9; // Decrease by 10%
                }
            }
        }

        // Normalize to 100%
        let total_weight: f64 = target_allocation.values().sum();
        for weight in target_allocation.values_mut() {
            *weight = (*weight / total_weight) * 100.0;
        }

        Ok(target_allocation)
    }

    fn calculate_portfolio_return(&self, allocation: &HashMap<String, f64>, expected_returns: &HashMap<String, f64>) -> f64 {
        allocation.iter()
            .map(|(asset, weight)| {
                let return_rate = expected_returns.get(asset).unwrap_or(&0.0);
                (weight / 100.0) * return_rate
            })
            .sum()
    }

    fn calculate_portfolio_risk(&self, allocation: &HashMap<String, f64>, risk_matrix: &HashMap<String, HashMap<String, f64>>) -> f64 {
        // Simplified portfolio risk calculation
        // In production, this would use proper covariance matrix calculations
        0.15 // 15% annualized volatility (simplified)
    }

    async fn generate_optimization_rationale(
        &self,
        current: &HashMap<String, f64>,
        target: &HashMap<String, f64>,
        predictions: &HashMap<String, AIPrediction>,
    ) -> Result<String> {
        let mut rationale = String::from("AI Portfolio Optimization Rationale:\n");
        
        for (asset, target_weight) in target {
            if let Some(current_weight) = current.get(asset) {
                let change = target_weight - current_weight;
                if change.abs() > 1.0 {
                    let direction = if change > 0.0 { "increase" } else { "decrease" };
                    rationale.push_str(&format!(
                        "- {} allocation to {} by {:.1}% based on AI prediction\n",
                        direction, asset, change.abs()
                    ));
                }
            }
        }

        Ok(rationale)
    }

    fn get_optimization_constraints(&self) -> Vec<OptimizationConstraint> {
        vec![
            OptimizationConstraint {
                constraint_type: "max_position_size".to_string(),
                asset: None,
                min_value: None,
                max_value: Some(self.max_position_size),
                description: "Maximum position size per asset".to_string(),
            },
            OptimizationConstraint {
                constraint_type: "min_liquidity".to_string(),
                asset: Some("USDC".to_string()),
                min_value: Some(self.liquidity_optimizer.min_liquidity_ratio),
                max_value: None,
                description: "Minimum liquidity requirement".to_string(),
            },
        ]
    }

    // Risk calculation methods (simplified implementations)
    async fn calculate_portfolio_var(&self, _assets: &HashMap<String, AssetAllocation>) -> Result<f64> {
        Ok(0.05) // 5% VaR (simplified)
    }

    async fn calculate_portfolio_cvar(&self, _assets: &HashMap<String, AssetAllocation>) -> Result<f64> {
        Ok(0.08) // 8% CVaR (simplified)
    }

    fn calculate_sharpe_ratio(&self, performance: &PerformanceMetrics) -> f64 {
        if performance.total_return_ytd > 0.0 {
            performance.total_return_ytd / 0.15 // Simplified: return / volatility
        } else {
            0.0
        }
    }

    async fn calculate_max_drawdown(&self) -> Result<f64> {
        Ok(0.12) // 12% max drawdown (simplified)
    }

    async fn calculate_correlation_risk(&self, _assets: &HashMap<String, AssetAllocation>) -> Result<f64> {
        Ok(0.3) // 30% correlation risk (simplified)
    }

    fn calculate_concentration_risk(&self, assets: &HashMap<String, AssetAllocation>) -> f64 {
        if assets.is_empty() {
            return 0.0;
        }

        // Calculate Herfindahl-Hirschman Index
        let hhi: f64 = assets.values()
            .map(|alloc| (alloc.current_percentage / 100.0).powi(2))
            .sum();

        // Convert to risk score (0-1)
        (hhi - (1.0 / assets.len() as f64)) / (1.0 - (1.0 / assets.len() as f64))
    }

    async fn calculate_liquidity_risk(&self, assets: &HashMap<String, AssetAllocation>) -> Result<f64> {
        if assets.is_empty() {
            return Ok(0.0);
        }

        let weighted_liquidity_score: f64 = assets.values()
            .map(|alloc| (alloc.current_percentage / 100.0) * (alloc.liquidity_score as f64 / 100.0))
            .sum();

        Ok(1.0 - weighted_liquidity_score) // Higher score = lower risk
    }

    fn calculate_overall_risk_score(&self, var: f64, correlation: f64, concentration: f64, liquidity: f64) -> u8 {
        let weighted_risk = (var * 0.3) + (correlation * 0.2) + (concentration * 0.3) + (liquidity * 0.2);
        (weighted_risk * 100.0).min(100.0) as u8
    }
}

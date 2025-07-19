use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep, Instant};
use tracing::{info, warn, error};

/// Comprehensive Monitoring and Metrics Service for RTF Infrastructure
/// PRD: "Real-time monitoring and prevention systems"
/// PRD: "Performance targets: <700ms API response time"
/// PRD: "Comprehensive audit trail logging"
pub struct MetricsService {
    metrics_store: RwLock<HashMap<String, MetricSeries>>,
    alert_rules: RwLock<HashMap<String, AlertRule>>,
    dashboards: RwLock<HashMap<String, Dashboard>>,
    performance_targets: PerformanceTargets,
    retention_policy: RetentionPolicy,
    real_time_monitoring: bool,
    audit_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub metric_name: String,
    pub metric_type: MetricType,
    pub data_points: Vec<DataPoint>,
    pub labels: HashMap<String, String>,
    pub unit: String,
    pub description: String,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,     // Monotonically increasing
    Gauge,       // Can go up and down
    Histogram,   // Distribution of values
    Summary,     // Quantiles over time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub rule_id: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
    pub enabled: bool,
    pub last_triggered: Option<i64>,
    pub trigger_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    PercentageChange,
    RateOfChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub dashboard_id: String,
    pub title: String,
    pub description: String,
    pub panels: Vec<DashboardPanel>,
    pub refresh_interval_seconds: u64,
    pub time_range: TimeRange,
    pub created_by: String,
    pub last_modified: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPanel {
    pub panel_id: String,
    pub title: String,
    pub panel_type: PanelType,
    pub metrics: Vec<String>,
    pub visualization_config: serde_json::Value,
    pub position: PanelPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanelType {
    LineChart,
    BarChart,
    Gauge,
    Table,
    Heatmap,
    SingleStat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub from: String, // e.g., "now-1h", "2024-01-01T00:00:00Z"
    pub to: String,   // e.g., "now", "2024-01-02T00:00:00Z"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    pub api_response_time_ms: u64,    // PRD: <700ms
    pub nav_computation_time_ms: u64,
    pub cross_chain_sync_time_ms: u64,
    pub uptime_percentage: f64,       // 99.9%
    pub error_rate_percentage: f64,   // <0.1%
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub high_resolution_days: u32,    // 7 days at 1-second resolution
    pub medium_resolution_days: u32,  // 30 days at 1-minute resolution
    pub low_resolution_days: u32,     // 365 days at 1-hour resolution
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub report_id: String,
    pub time_period: TimeRange,
    pub api_performance: APIPerformanceMetrics,
    pub system_performance: SystemPerformanceMetrics,
    pub business_metrics: BusinessMetrics,
    pub sla_compliance: SLACompliance,
    pub generated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIPerformanceMetrics {
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub p99_response_time_ms: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,
    pub success_rate: f64,
    pub slowest_endpoints: Vec<EndpointMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetric {
    pub endpoint: String,
    pub method: String,
    pub average_response_time_ms: f64,
    pub request_count: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPerformanceMetrics {
    pub cpu_usage_percentage: f64,
    pub memory_usage_percentage: f64,
    pub disk_usage_percentage: f64,
    pub network_io_mbps: f64,
    pub database_connections: u32,
    pub cache_hit_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub total_assets_under_management: u64,
    pub daily_nav_updates: u64,
    pub cross_chain_transactions: u64,
    pub active_vaults: u32,
    pub total_users: u32,
    pub redemption_requests: u64,
    pub compliance_checks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLACompliance {
    pub uptime_percentage: f64,
    pub api_response_time_sla_met: bool,
    pub nav_computation_sla_met: bool,
    pub cross_chain_sync_sla_met: bool,
    pub overall_sla_score: f64,
}

impl MetricsService {
    /// Initialize Metrics Service
    pub async fn new() -> Result<Self> {
        info!("📊 Initializing Comprehensive Metrics Service");
        
        let service = Self {
            metrics_store: RwLock::new(HashMap::new()),
            alert_rules: RwLock::new(HashMap::new()),
            dashboards: RwLock::new(HashMap::new()),
            performance_targets: PerformanceTargets {
                api_response_time_ms: 700,    // PRD requirement
                nav_computation_time_ms: 5000, // 5 seconds
                cross_chain_sync_time_ms: 30000, // 30 seconds
                uptime_percentage: 99.9,
                error_rate_percentage: 0.1,
            },
            retention_policy: RetentionPolicy {
                high_resolution_days: 7,
                medium_resolution_days: 30,
                low_resolution_days: 365,
            },
            real_time_monitoring: true,
            audit_logging: true,
        };

        // Initialize default metrics
        service.initialize_default_metrics().await?;
        
        // Initialize alert rules
        service.initialize_alert_rules().await?;
        
        // Initialize dashboards
        service.initialize_dashboards().await?;

        info!("✅ Metrics Service initialized with real-time monitoring enabled");
        Ok(service)
    }

    /// PRD: Record metric with real-time monitoring
    /// PRD: "Real-time monitoring and prevention systems"
    pub async fn record_metric(
        &self,
        metric_name: String,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        
        let data_point = DataPoint {
            timestamp,
            value,
            labels: labels.clone(),
        };

        // Store metric
        {
            let mut metrics = self.metrics_store.write().await;
            if let Some(series) = metrics.get_mut(&metric_name) {
                series.data_points.push(data_point);
                series.last_updated = timestamp;
                
                // Apply retention policy
                self.apply_retention_policy(series);
            } else {
                // Create new metric series
                let series = MetricSeries {
                    metric_name: metric_name.clone(),
                    metric_type: MetricType::Gauge, // Default type
                    data_points: vec![data_point],
                    labels,
                    unit: "".to_string(),
                    description: "".to_string(),
                    last_updated: timestamp,
                };
                metrics.insert(metric_name.clone(), series);
            }
        }

        // Check alert rules if real-time monitoring is enabled
        if self.real_time_monitoring {
            self.check_alert_rules(&metric_name, value).await?;
        }

        Ok(())
    }

    /// PRD: Track API performance against <700ms target
    /// PRD: "Performance targets: <700ms API response time"
    pub async fn record_api_performance(
        &self,
        endpoint: String,
        method: String,
        response_time_ms: u64,
        status_code: u16,
    ) -> Result<()> {
        let labels = HashMap::from([
            ("endpoint".to_string(), endpoint),
            ("method".to_string(), method),
            ("status_code".to_string(), status_code.to_string()),
        ]);

        // Record response time
        self.record_metric(
            "api_response_time_ms".to_string(),
            response_time_ms as f64,
            labels.clone(),
        ).await?;

        // Record request count
        self.record_metric(
            "api_requests_total".to_string(),
            1.0,
            labels.clone(),
        ).await?;

        // Record error if status code indicates error
        if status_code >= 400 {
            self.record_metric(
                "api_errors_total".to_string(),
                1.0,
                labels,
            ).await?;
        }

        // Check if response time exceeds SLA
        if response_time_ms > self.performance_targets.api_response_time_ms {
            warn!("🚨 API response time SLA violation: {}ms > {}ms", 
                  response_time_ms, self.performance_targets.api_response_time_ms);
        }

        Ok(())
    }

    /// PRD: Generate comprehensive performance report
    /// PRD: "Comprehensive audit trail logging"
    pub async fn generate_performance_report(
        &self,
        time_range: TimeRange,
    ) -> Result<PerformanceReport> {
        info!("📈 Generating performance report for period: {:?}", time_range);

        let report_id = format!("report_{}", chrono::Utc::now().timestamp());

        // Calculate API performance metrics
        let api_performance = self.calculate_api_performance_metrics(&time_range).await?;
        
        // Calculate system performance metrics
        let system_performance = self.calculate_system_performance_metrics(&time_range).await?;
        
        // Calculate business metrics
        let business_metrics = self.calculate_business_metrics(&time_range).await?;
        
        // Calculate SLA compliance
        let sla_compliance = self.calculate_sla_compliance(&api_performance, &system_performance).await?;

        let report = PerformanceReport {
            report_id,
            time_period: time_range,
            api_performance,
            system_performance,
            business_metrics,
            sla_compliance,
            generated_at: chrono::Utc::now().timestamp(),
        };

        info!("✅ Performance report generated - Overall SLA score: {:.2}%", 
              report.sla_compliance.overall_sla_score);

        Ok(report)
    }

    /// Start real-time monitoring loop
    pub async fn start_monitoring(&self) -> Result<()> {
        info!("🔄 Starting real-time monitoring loop");

        loop {
            // Collect system metrics
            self.collect_system_metrics().await?;
            
            // Check all alert rules
            self.check_all_alert_rules().await?;
            
            // Clean up old metrics based on retention policy
            self.cleanup_old_metrics().await?;
            
            // Sleep for monitoring interval
            sleep(Duration::from_secs(10)).await;
        }
    }

    // Private helper methods
    async fn initialize_default_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics_store.write().await;
        
        // API metrics
        metrics.insert("api_response_time_ms".to_string(), MetricSeries {
            metric_name: "api_response_time_ms".to_string(),
            metric_type: MetricType::Histogram,
            data_points: Vec::new(),
            labels: HashMap::new(),
            unit: "milliseconds".to_string(),
            description: "API response time in milliseconds".to_string(),
            last_updated: 0,
        });

        // NAV computation metrics
        metrics.insert("nav_computation_time_ms".to_string(), MetricSeries {
            metric_name: "nav_computation_time_ms".to_string(),
            metric_type: MetricType::Gauge,
            data_points: Vec::new(),
            labels: HashMap::new(),
            unit: "milliseconds".to_string(),
            description: "NAV computation time".to_string(),
            last_updated: 0,
        });

        // Cross-chain sync metrics
        metrics.insert("cross_chain_sync_time_ms".to_string(), MetricSeries {
            metric_name: "cross_chain_sync_time_ms".to_string(),
            metric_type: MetricType::Gauge,
            data_points: Vec::new(),
            labels: HashMap::new(),
            unit: "milliseconds".to_string(),
            description: "Cross-chain synchronization time".to_string(),
            last_updated: 0,
        });

        // Business metrics
        metrics.insert("total_aum".to_string(), MetricSeries {
            metric_name: "total_aum".to_string(),
            metric_type: MetricType::Gauge,
            data_points: Vec::new(),
            labels: HashMap::new(),
            unit: "USD".to_string(),
            description: "Total Assets Under Management".to_string(),
            last_updated: 0,
        });

        Ok(())
    }

    async fn initialize_alert_rules(&self) -> Result<()> {
        let mut rules = self.alert_rules.write().await;

        // API response time alert
        rules.insert("api_response_time_high".to_string(), AlertRule {
            rule_id: "api_response_time_high".to_string(),
            metric_name: "api_response_time_ms".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: self.performance_targets.api_response_time_ms as f64,
            duration_seconds: 60,
            severity: AlertSeverity::Warning,
            notification_channels: vec!["email".to_string(), "slack".to_string()],
            enabled: true,
            last_triggered: None,
            trigger_count: 0,
        });

        // Critical API response time alert
        rules.insert("api_response_time_critical".to_string(), AlertRule {
            rule_id: "api_response_time_critical".to_string(),
            metric_name: "api_response_time_ms".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: (self.performance_targets.api_response_time_ms * 2) as f64,
            duration_seconds: 30,
            severity: AlertSeverity::Critical,
            notification_channels: vec!["email".to_string(), "slack".to_string(), "pagerduty".to_string()],
            enabled: true,
            last_triggered: None,
            trigger_count: 0,
        });

        // NAV computation time alert
        rules.insert("nav_computation_slow".to_string(), AlertRule {
            rule_id: "nav_computation_slow".to_string(),
            metric_name: "nav_computation_time_ms".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: self.performance_targets.nav_computation_time_ms as f64,
            duration_seconds: 120,
            severity: AlertSeverity::Warning,
            notification_channels: vec!["email".to_string()],
            enabled: true,
            last_triggered: None,
            trigger_count: 0,
        });

        Ok(())
    }

    async fn initialize_dashboards(&self) -> Result<()> {
        let mut dashboards = self.dashboards.write().await;

        // Main RTF Dashboard
        dashboards.insert("rtf_main".to_string(), Dashboard {
            dashboard_id: "rtf_main".to_string(),
            title: "RTF Infrastructure Overview".to_string(),
            description: "Main dashboard for RTF infrastructure monitoring".to_string(),
            panels: vec![
                DashboardPanel {
                    panel_id: "api_response_time".to_string(),
                    title: "API Response Time".to_string(),
                    panel_type: PanelType::LineChart,
                    metrics: vec!["api_response_time_ms".to_string()],
                    visualization_config: serde_json::json!({"yAxis": {"max": 1000}}),
                    position: PanelPosition { x: 0, y: 0, width: 6, height: 4 },
                },
                DashboardPanel {
                    panel_id: "total_aum".to_string(),
                    title: "Total AUM".to_string(),
                    panel_type: PanelType::SingleStat,
                    metrics: vec!["total_aum".to_string()],
                    visualization_config: serde_json::json!({"unit": "USD", "decimals": 0}),
                    position: PanelPosition { x: 6, y: 0, width: 3, height: 2 },
                },
                DashboardPanel {
                    panel_id: "nav_computation_time".to_string(),
                    title: "NAV Computation Time".to_string(),
                    panel_type: PanelType::Gauge,
                    metrics: vec!["nav_computation_time_ms".to_string()],
                    visualization_config: serde_json::json!({"max": 10000, "thresholds": [5000, 8000]}),
                    position: PanelPosition { x: 9, y: 0, width: 3, height: 2 },
                },
            ],
            refresh_interval_seconds: 30,
            time_range: TimeRange {
                from: "now-1h".to_string(),
                to: "now".to_string(),
            },
            created_by: "system".to_string(),
            last_modified: chrono::Utc::now().timestamp(),
        });

        Ok(())
    }

    async fn check_alert_rules(&self, metric_name: &str, value: f64) -> Result<()> {
        let rules = self.alert_rules.read().await;
        
        for rule in rules.values() {
            if rule.metric_name == metric_name && rule.enabled {
                let triggered = match rule.condition {
                    AlertCondition::GreaterThan => value > rule.threshold,
                    AlertCondition::LessThan => value < rule.threshold,
                    AlertCondition::Equals => (value - rule.threshold).abs() < 0.001,
                    AlertCondition::NotEquals => (value - rule.threshold).abs() >= 0.001,
                    _ => false, // Other conditions need historical data
                };

                if triggered {
                    self.trigger_alert(rule, value).await?;
                }
            }
        }

        Ok(())
    }

    async fn trigger_alert(&self, rule: &AlertRule, value: f64) -> Result<()> {
        warn!("🚨 Alert triggered: {} - Value: {}, Threshold: {}", 
              rule.rule_id, value, rule.threshold);

        // Update alert rule state
        {
            let mut rules = self.alert_rules.write().await;
            if let Some(mut_rule) = rules.get_mut(&rule.rule_id) {
                mut_rule.last_triggered = Some(chrono::Utc::now().timestamp());
                mut_rule.trigger_count += 1;
            }
        }

        // Send notifications (simplified)
        for channel in &rule.notification_channels {
            info!("📢 Sending alert to channel: {}", channel);
            // TODO: Implement actual notification sending
        }

        Ok(())
    }

    async fn check_all_alert_rules(&self) -> Result<()> {
        // This would check rules that require historical data analysis
        Ok(())
    }

    async fn collect_system_metrics(&self) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        
        // Simulate system metric collection
        // In production, these would be actual system metrics
        
        // CPU usage
        self.record_metric(
            "system_cpu_usage_percent".to_string(),
            45.0, // Simulated 45% CPU usage
            HashMap::new(),
        ).await?;

        // Memory usage
        self.record_metric(
            "system_memory_usage_percent".to_string(),
            60.0, // Simulated 60% memory usage
            HashMap::new(),
        ).await?;

        // Disk usage
        self.record_metric(
            "system_disk_usage_percent".to_string(),
            30.0, // Simulated 30% disk usage
            HashMap::new(),
        ).await?;

        Ok(())
    }

    async fn cleanup_old_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics_store.write().await;
        let current_time = chrono::Utc::now().timestamp();
        
        for series in metrics.values_mut() {
            // Remove data points older than retention policy
            let retention_seconds = self.retention_policy.low_resolution_days as i64 * 86400;
            series.data_points.retain(|dp| current_time - dp.timestamp < retention_seconds);
        }

        Ok(())
    }

    fn apply_retention_policy(&self, series: &mut MetricSeries) {
        // Keep only the most recent data points based on retention policy
        let max_points = 10000; // Configurable limit
        if series.data_points.len() > max_points {
            series.data_points.drain(0..series.data_points.len() - max_points);
        }
    }

    async fn calculate_api_performance_metrics(&self, _time_range: &TimeRange) -> Result<APIPerformanceMetrics> {
        // Simplified calculation - in production would analyze actual data
        Ok(APIPerformanceMetrics {
            average_response_time_ms: 450.0,
            p95_response_time_ms: 650.0,
            p99_response_time_ms: 850.0,
            requests_per_second: 100.0,
            error_rate: 0.05,
            success_rate: 99.95,
            slowest_endpoints: vec![
                EndpointMetric {
                    endpoint: "/api/v1/nav/compute".to_string(),
                    method: "POST".to_string(),
                    average_response_time_ms: 1200.0,
                    request_count: 1000,
                    error_count: 2,
                },
            ],
        })
    }

    async fn calculate_system_performance_metrics(&self, _time_range: &TimeRange) -> Result<SystemPerformanceMetrics> {
        Ok(SystemPerformanceMetrics {
            cpu_usage_percentage: 45.0,
            memory_usage_percentage: 60.0,
            disk_usage_percentage: 30.0,
            network_io_mbps: 50.0,
            database_connections: 25,
            cache_hit_rate: 95.0,
        })
    }

    async fn calculate_business_metrics(&self, _time_range: &TimeRange) -> Result<BusinessMetrics> {
        Ok(BusinessMetrics {
            total_assets_under_management: 10_000_000,
            daily_nav_updates: 50,
            cross_chain_transactions: 200,
            active_vaults: 25,
            total_users: 1000,
            redemption_requests: 30,
            compliance_checks: 150,
        })
    }

    async fn calculate_sla_compliance(&self, api: &APIPerformanceMetrics, _system: &SystemPerformanceMetrics) -> Result<SLACompliance> {
        let api_sla_met = api.average_response_time_ms < self.performance_targets.api_response_time_ms as f64;
        let nav_sla_met = true; // Simplified
        let cross_chain_sla_met = true; // Simplified
        
        let overall_score = if api_sla_met && nav_sla_met && cross_chain_sla_met {
            99.9
        } else {
            95.0
        };

        Ok(SLACompliance {
            uptime_percentage: 99.95,
            api_response_time_sla_met: api_sla_met,
            nav_computation_sla_met: nav_sla_met,
            cross_chain_sync_sla_met: cross_chain_sla_met,
            overall_sla_score: overall_score,
        })
    }
}

//! Analysis module for telemetry data.
//!
//! This module provides functionality for analyzing telemetry data,
//! identifying patterns, detecting anomalies, and generating insights.

use std::collections::{HashMap, HashSet};
use anyhow::{Result, Context, anyhow, bail};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use statrs::statistics::{Data, Distribution, Mean, Variance, OrderStatistics};
use statrs::distribution::Normal;

use super::{TelemetryDataPoint, TelemetryCategory, TelemetryValue};

/// Telemetry analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryAnalysis {
    /// Timestamp when the analysis was performed.
    pub timestamp: DateTime<Utc>,
    
    /// Summary statistics for numeric metrics.
    pub statistics: HashMap<String, MetricStatistics>,
    
    /// Detected anomalies.
    pub anomalies: Vec<Anomaly>,
    
    /// Detected trends.
    pub trends: Vec<Trend>,
    
    /// Correlations between metrics.
    pub correlations: Vec<Correlation>,
    
    /// Insights derived from the analysis.
    pub insights: Vec<Insight>,
}

/// Statistics for a numeric metric.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStatistics {
    /// Name of the metric.
    pub name: String,
    
    /// Category of the metric.
    pub category: TelemetryCategory,
    
    /// Number of data points.
    pub count: usize,
    
    /// Minimum value.
    pub min: f64,
    
    /// Maximum value.
    pub max: f64,
    
    /// Mean value.
    pub mean: f64,
    
    /// Median value.
    pub median: f64,
    
    /// Standard deviation.
    pub std_dev: f64,
    
    /// 25th percentile.
    pub percentile_25: f64,
    
    /// 75th percentile.
    pub percentile_75: f64,
    
    /// Most recent value.
    pub latest_value: f64,
    
    /// Change from previous period (percentage).
    pub change_percent: Option<f64>,
}

/// Detected anomaly in telemetry data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Name of the metric with the anomaly.
    pub metric_name: String,
    
    /// Category of the metric.
    pub category: TelemetryCategory,
    
    /// Timestamp when the anomaly occurred.
    pub timestamp: DateTime<Utc>,
    
    /// Anomalous value.
    pub value: f64,
    
    /// Expected value range.
    pub expected_range: (f64, f64),
    
    /// Severity of the anomaly (0.0 to 1.0).
    pub severity: f64,
    
    /// Description of the anomaly.
    pub description: String,
}

/// Detected trend in telemetry data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trend {
    /// Name of the metric with the trend.
    pub metric_name: String,
    
    /// Category of the metric.
    pub category: TelemetryCategory,
    
    /// Direction of the trend (positive or negative).
    pub direction: TrendDirection,
    
    /// Magnitude of the trend (percentage change).
    pub magnitude: f64,
    
    /// Duration of the trend.
    pub duration: Duration,
    
    /// Description of the trend.
    pub description: String,
}

/// Direction of a trend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Increasing trend.
    Increasing,
    
    /// Decreasing trend.
    Decreasing,
    
    /// Stable trend.
    Stable,
}

/// Correlation between metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correlation {
    /// Name of the first metric.
    pub metric1_name: String,
    
    /// Category of the first metric.
    pub metric1_category: TelemetryCategory,
    
    /// Name of the second metric.
    pub metric2_name: String,
    
    /// Category of the second metric.
    pub metric2_category: TelemetryCategory,
    
    /// Correlation coefficient (-1.0 to 1.0).
    pub coefficient: f64,
    
    /// Strength of the correlation.
    pub strength: CorrelationStrength,
    
    /// Description of the correlation.
    pub description: String,
}

/// Strength of a correlation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrelationStrength {
    /// Strong correlation.
    Strong,
    
    /// Moderate correlation.
    Moderate,
    
    /// Weak correlation.
    Weak,
    
    /// No correlation.
    None,
}

/// Insight derived from telemetry analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Type of insight.
    pub insight_type: InsightType,
    
    /// Metrics related to the insight.
    pub related_metrics: Vec<String>,
    
    /// Severity of the insight (0.0 to 1.0).
    pub severity: f64,
    
    /// Description of the insight.
    pub description: String,
    
    /// Recommended actions.
    pub recommendations: Vec<String>,
}

/// Type of insight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsightType {
    /// Performance issue.
    PerformanceIssue,
    
    /// Resource constraint.
    ResourceConstraint,
    
    /// Hardware issue.
    HardwareIssue,
    
    /// Usage pattern.
    UsagePattern,
    
    /// Security concern.
    SecurityConcern,
    
    /// System health.
    SystemHealth,
}

/// Analyze telemetry data.
///
/// This function analyzes telemetry data to identify patterns, detect
/// anomalies, and generate insights.
///
/// # Arguments
///
/// * `data` - Telemetry data points to analyze
///
/// # Returns
///
/// Analysis results.
pub fn analyze_telemetry(data: &[TelemetryDataPoint]) -> Result<TelemetryAnalysis> {
    // Group data by metric name and category
    let grouped_data = group_data_by_metric(data);
    
    // Calculate statistics for each metric
    let statistics = calculate_statistics(&grouped_data)?;
    
    // Detect anomalies
    let anomalies = detect_anomalies(&grouped_data, &statistics)?;
    
    // Detect trends
    let trends = detect_trends(&grouped_data)?;
    
    // Calculate correlations
    let correlations = calculate_correlations(&grouped_data)?;
    
    // Generate insights
    let insights = generate_insights(&statistics, &anomalies, &trends, &correlations)?;
    
    Ok(TelemetryAnalysis {
        timestamp: Utc::now(),
        statistics,
        anomalies,
        trends,
        correlations,
        insights,
    })
}

/// Group telemetry data by metric name and category.
///
/// # Arguments
///
/// * `data` - Telemetry data points to group
///
/// # Returns
///
/// A map of (metric name, category) to a vector of (timestamp, value) pairs.
fn group_data_by_metric(data: &[TelemetryDataPoint]) -> HashMap<(String, TelemetryCategory), Vec<(DateTime<Utc>, f64)>> {
    let mut grouped_data = HashMap::new();
    
    for point in data {
        // Extract numeric value if possible
        if let Some(value) = extract_numeric_value(&point.value) {
            let key = (point.name.clone(), point.category);
            let entry = grouped_data.entry(key).or_insert_with(Vec::new);
            entry.push((point.timestamp, value));
        }
    }
    
    // Sort each group by timestamp
    for values in grouped_data.values_mut() {
        values.sort_by(|a, b| a.0.cmp(&b.0));
    }
    
    grouped_data
}

/// Extract a numeric value from a telemetry value.
///
/// # Arguments
///
/// * `value` - Telemetry value to extract from
///
/// # Returns
///
/// Extracted numeric value, if possible.
fn extract_numeric_value(value: &TelemetryValue) -> Option<f64> {
    match value {
        TelemetryValue::Integer(i) => Some(*i as f64),
        TelemetryValue::Float(f) => Some(*f),
        TelemetryValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Calculate statistics for grouped telemetry data.
///
/// # Arguments
///
/// * `grouped_data` - Grouped telemetry data
///
/// # Returns
///
/// A map of metric names to statistics.
fn calculate_statistics(
    grouped_data: &HashMap<(String, TelemetryCategory), Vec<(DateTime<Utc>, f64)>>
) -> Result<HashMap<String, MetricStatistics>> {
    let mut statistics = HashMap::new();
    
    for ((name, category), values) in grouped_data {
        if values.is_empty() {
            continue;
        }
        
        // Extract values
        let numeric_values: Vec<f64> = values.iter().map(|(_, v)| *v).collect();
        
        // Create a Data object for statistical calculations
        let data = Data::new(numeric_values.clone());
        
        // Calculate statistics
        let count = numeric_values.len();
        let min = data.min();
        let max = data.max();
        let mean = data.mean();
        let median = data.median();
        let std_dev = data.std_dev();
        let percentile_25 = data.percentile(25.0);
        let percentile_75 = data.percentile(75.0);
        let latest_value = numeric_values.last().cloned().unwrap_or(0.0);
        
        // Calculate change percentage if there are at least 2 values
        let change_percent = if count >= 2 {
            let previous_value = numeric_values[count - 2];
            if previous_value != 0.0 {
                Some((latest_value - previous_value) / previous_value * 100.0)
            } else {
                None
            }
        } else {
            None
        };
        
        // Create statistics object
        let stats = MetricStatistics {
            name: name.clone(),
            category: *category,
            count,
            min,
            max,
            mean,
            median,
            std_dev,
            percentile_25,
            percentile_75,
            latest_value,
            change_percent,
        };
        
        statistics.insert(name.clone(), stats);
    }
    
    Ok(statistics)
}

/// Detect anomalies in telemetry data.
///
/// # Arguments
///
/// * `grouped_data` - Grouped telemetry data
/// * `statistics` - Statistics for each metric
///
/// # Returns
///
/// A vector of detected anomalies.
fn detect_anomalies(
    grouped_data: &HashMap<(String, TelemetryCategory), Vec<(DateTime<Utc>, f64)>>,
    statistics: &HashMap<String, MetricStatistics>,
) -> Result<Vec<Anomaly>> {
    let mut anomalies = Vec::new();
    
    for ((name, category), values) in grouped_data {
        if values.len() < 10 {
            // Not enough data for anomaly detection
            continue;
        }
        
        if let Some(stats) = statistics.get(name) {
            // Use Z-score method for anomaly detection
            let threshold = 3.0; // Values more than 3 standard deviations from the mean are anomalies
            
            for (timestamp, value) in values {
                if stats.std_dev > 0.0 {
                    let z_score = (value - stats.mean) / stats.std_dev;
                    
                    if z_score.abs() > threshold {
                        // This is an anomaly
                        let expected_range = (
                            stats.mean - threshold * stats.std_dev,
                            stats.mean + threshold * stats.std_dev,
                        );
                        
                        let severity = (z_score.abs() - threshold) / 2.0;
                        let severity = severity.min(1.0);
                        
                        let direction = if z_score > 0.0 { "high" } else { "low" };
                        let description = format!(
                            "Anomalous {} value of {:.2} detected (expected range: {:.2} to {:.2})",
                            direction, value, expected_range.0, expected_range.1
                        );
                        
                        anomalies.push(Anomaly {
                            metric_name: name.clone(),
                            category: *category,
                            timestamp: *timestamp,
                            value: *value,
                            expected_range,
                            severity,
                            description,
                        });
                    }
                }
            }
        }
    }
    
    // Sort anomalies by severity (highest first)
    anomalies.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(anomalies)
}

/// Detect trends in telemetry data.
///
/// # Arguments
///
/// * `grouped_data` - Grouped telemetry data
///
/// # Returns
///
/// A vector of detected trends.
fn detect_trends(
    grouped_data: &HashMap<(String, TelemetryCategory), Vec<(DateTime<Utc>, f64)>>
) -> Result<Vec<Trend>> {
    let mut trends = Vec::new();
    
    for ((name, category), values) in grouped_data {
        if values.len() < 10 {
            // Not enough data for trend detection
            continue;
        }
        
        // Use linear regression to detect trends
        let timestamps: Vec<i64> = values.iter().map(|(t, _)| t.timestamp()).collect();
        let numeric_values: Vec<f64> = values.iter().map(|(_, v)| *v).collect();
        
        let (slope, _) = linear_regression(&timestamps, &numeric_values)?;
        
        // Determine trend direction and magnitude
        let direction = if slope > 0.001 {
            TrendDirection::Increasing
        } else if slope < -0.001 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };
        
        // Only report non-stable trends
        if direction != TrendDirection::Stable {
            // Calculate duration
            let start_time = values.first().map(|(t, _)| *t).unwrap_or_else(Utc::now);
            let end_time = values.last().map(|(t, _)| *t).unwrap_or_else(Utc::now);
            let duration = end_time - start_time;
            
            // Calculate magnitude (percentage change over the period)
            let first_value = values.first().map(|(_, v)| *v).unwrap_or(0.0);
            let last_value = values.last().map(|(_, v)| *v).unwrap_or(0.0);
            
            let magnitude = if first_value != 0.0 {
                ((last_value - first_value) / first_value) * 100.0
            } else {
                0.0
            };
            
            let direction_str = match direction {
                TrendDirection::Increasing => "increasing",
                TrendDirection::Decreasing => "decreasing",
                TrendDirection::Stable => "stable",
            };
            
            let description = format!(
                "{} is {} by {:.2}% over {} hours",
                name,
                direction_str,
                magnitude.abs(),
                duration.num_hours()
            );
            
            trends.push(Trend {
                metric_name: name.clone(),
                category: *category,
                direction,
                magnitude: magnitude.abs(),
                duration,
                description,
            });
        }
    }
    
    // Sort trends by magnitude (highest first)
    trends.sort_by(|a, b| b.magnitude.partial_cmp(&a.magnitude).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(trends)
}

/// Calculate correlations between metrics.
///
/// # Arguments
///
/// * `grouped_data` - Grouped telemetry data
///
/// # Returns
///
/// A vector of detected correlations.
fn calculate_correlations(
    grouped_data: &HashMap<(String, TelemetryCategory), Vec<(DateTime<Utc>, f64)>>
) -> Result<Vec<Correlation>> {
    let mut correlations = Vec::new();
    
    // Get all metric names
    let metrics: Vec<(String, TelemetryCategory)> = grouped_data.keys().cloned().collect();
    
    // Calculate correlations between all pairs of metrics
    for i in 0..metrics.len() {
        for j in i+1..metrics.len() {
            let metric1 = &metrics[i];
            let metric2 = &metrics[j];
            
            let values1 = &grouped_data[metric1];
            let values2 = &grouped_data[metric2];
            
            if values1.len() < 10 || values2.len() < 10 {
                // Not enough data for correlation
                continue;
            }
            
            // Align timestamps and values
            let aligned_values = align_time_series(values1, values2);
            
            if aligned_values.len() < 10 {
                // Not enough aligned data points
                continue;
            }
            
            // Extract aligned values
            let x: Vec<f64> = aligned_values.iter().map(|(x, _)| *x).collect();
            let y: Vec<f64> = aligned_values.iter().map(|(_, y)| *y).collect();
            
            // Calculate Pearson correlation coefficient
            let coefficient = pearson_correlation(&x, &y)?;
            
            // Determine correlation strength
            let strength = if coefficient.abs() > 0.7 {
                CorrelationStrength::Strong
            } else if coefficient.abs() > 0.4 {
                CorrelationStrength::Moderate
            } else if coefficient.abs() > 0.2 {
                CorrelationStrength::Weak
            } else {
                CorrelationStrength::None
            };
            
            // Only report non-zero correlations
            if strength != CorrelationStrength::None {
                let direction = if coefficient > 0.0 { "positive" } else { "negative" };
                let strength_str = match strength {
                    CorrelationStrength::Strong => "strong",
                    CorrelationStrength::Moderate => "moderate",
                    CorrelationStrength::Weak => "weak",
                    CorrelationStrength::None => "no",
                };
                
                let description = format!(
                    "{} {} correlation between {} and {} (coefficient: {:.2})",
                    strength_str,
                    direction,
                    metric1.0,
                    metric2.0,
                    coefficient
                );
                
                correlations.push(Correlation {
                    metric1_name: metric1.0.clone(),
                    metric1_category: metric1.1,
                    metric2_name: metric2.0.clone(),
                    metric2_category: metric2.1,
                    coefficient,
                    strength,
                    description,
                });
            }
        }
    }
    
    // Sort correlations by absolute coefficient (highest first)
    correlations.sort_by(|a, b| {
        b.coefficient.abs().partial_cmp(&a.coefficient.abs()).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    Ok(correlations)
}

/// Generate insights from analysis results.
///
/// # Arguments
///
/// * `statistics` - Statistics for each metric
/// * `anomalies` - Detected anomalies
/// * `trends` - Detected trends
/// * `correlations` - Detected correlations
///
/// # Returns
///
/// A vector of generated insights.
fn generate_insights(
    statistics: &HashMap<String, MetricStatistics>,
    anomalies: &[Anomaly],
    trends: &[Trend],
    correlations: &[Correlation],
) -> Result<Vec<Insight>> {
    let mut insights = Vec::new();
    
    // Generate insights from anomalies
    for anomaly in anomalies {
        if anomaly.severity > 0.5 {
            // High severity anomaly
            let insight_type = match anomaly.category {
                TelemetryCategory::System => InsightType::SystemHealth,
                TelemetryCategory::Hardware => InsightType::HardwareIssue,
                TelemetryCategory::Application => InsightType::PerformanceIssue,
                TelemetryCategory::Network => InsightType::PerformanceIssue,
                TelemetryCategory::Error => InsightType::SystemHealth,
                TelemetryCategory::UserInteraction => InsightType::UsagePattern,
                TelemetryCategory::Custom(_) => InsightType::SystemHealth,
            };
            
            let description = format!(
                "Critical anomaly detected in {}: {}",
                anomaly.metric_name,
                anomaly.description
            );
            
            let recommendations = generate_recommendations(insight_type, &anomaly.metric_name, anomaly.value);
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![anomaly.metric_name.clone()],
                severity: anomaly.severity,
                description,
                recommendations,
            });
        }
    }
    
    // Generate insights from trends
    for trend in trends {
        if trend.magnitude > 20.0 {
            // Significant trend
            let insight_type = match trend.category {
                TelemetryCategory::System => InsightType::SystemHealth,
                TelemetryCategory::Hardware => InsightType::HardwareIssue,
                TelemetryCategory::Application => InsightType::PerformanceIssue,
                TelemetryCategory::Network => InsightType::PerformanceIssue,
                TelemetryCategory::Error => InsightType::SystemHealth,
                TelemetryCategory::UserInteraction => InsightType::UsagePattern,
                TelemetryCategory::Custom(_) => InsightType::SystemHealth,
            };
            
            let severity = trend.magnitude / 100.0;
            let severity = severity.min(1.0);
            
            let description = format!(
                "Significant trend detected: {}",
                trend.description
            );
            
            let latest_value = statistics.get(&trend.metric_name)
                .map(|s| s.latest_value)
                .unwrap_or(0.0);
            
            let recommendations = generate_recommendations(insight_type, &trend.metric_name, latest_value);
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![trend.metric_name.clone()],
                severity,
                description,
                recommendations,
            });
        }
    }
    
    // Generate insights from correlations
    for correlation in correlations {
        if correlation.strength == CorrelationStrength::Strong {
            // Strong correlation
            let insight_type = match (correlation.metric1_category, correlation.metric2_category) {
                (TelemetryCategory::System, TelemetryCategory::Hardware) => InsightType::HardwareIssue,
                (TelemetryCategory::Hardware, TelemetryCategory::System) => InsightType::HardwareIssue,
                (TelemetryCategory::System, TelemetryCategory::Application) => InsightType::PerformanceIssue,
                (TelemetryCategory::Application, TelemetryCategory::System) => InsightType::PerformanceIssue,
                (TelemetryCategory::System, TelemetryCategory::Network) => InsightType::PerformanceIssue,
                (TelemetryCategory::Network, TelemetryCategory::System) => InsightType::PerformanceIssue,
                (TelemetryCategory::Hardware, TelemetryCategory::Application) => InsightType::PerformanceIssue,
                (TelemetryCategory::Application, TelemetryCategory::Hardware) => InsightType::PerformanceIssue,
                (TelemetryCategory::Hardware, TelemetryCategory::Network) => InsightType::PerformanceIssue,
                (TelemetryCategory::Network, TelemetryCategory::Hardware) => InsightType::PerformanceIssue,
                (TelemetryCategory::Application, TelemetryCategory::Network) => InsightType::PerformanceIssue,
                (TelemetryCategory::Network, TelemetryCategory::Application) => InsightType::PerformanceIssue,
                (TelemetryCategory::Error, _) => InsightType::SystemHealth,
                (_, TelemetryCategory::Error) => InsightType::SystemHealth,
                (TelemetryCategory::UserInteraction, _) => InsightType::UsagePattern,
                (_, TelemetryCategory::UserInteraction) => InsightType::UsagePattern,
                _ => InsightType::SystemHealth,
            };
            
            let severity = correlation.coefficient.abs();
            
            let description = format!(
                "Strong correlation detected: {}",
                correlation.description
            );
            
            let recommendations = vec![
                format!("Monitor both {} and {} together", correlation.metric1_name, correlation.metric2_name),
                format!("Investigate potential causal relationship between {} and {}", correlation.metric1_name, correlation.metric2_name),
            ];
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![correlation.metric1_name.clone(), correlation.metric2_name.clone()],
                severity,
                description,
                recommendations,
            });
        }
    }
    
    // Generate insights from statistics
    for (name, stats) in statistics {
        // Check for resource constraints
        if (name.contains("cpu") || name.contains("memory") || name.contains("disk")) && stats.latest_value > 90.0 {
            let insight_type = InsightType::ResourceConstraint;
            
            let severity = (stats.latest_value - 90.0) / 10.0;
            let severity = severity.min(1.0);
            
            let resource_type = if name.contains("cpu") {
                "CPU"
            } else if name.contains("memory") {
                "Memory"
            } else {
                "Disk"
            };
            
            let description = format!(
                "{} usage is critically high at {:.2}%",
                resource_type,
                stats.latest_value
            );
            
            let recommendations = vec![
                format!("Reduce {} usage by closing unnecessary applications", resource_type),
                format!("Investigate processes consuming high {}", resource_type),
                "Consider upgrading hardware if this is a recurring issue".to_string(),
            ];
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![name.clone()],
                severity,
                description,
                recommendations,
            });
        }
        
        // Check for performance issues
        if (name.contains("fps") || name.contains("frame_time")) && 
           ((name.contains("fps") && stats.latest_value < 60.0) || 
            (name.contains("frame_time") && stats.latest_value > 16.0)) {
            let insight_type = InsightType::PerformanceIssue;
            
            let severity = if name.contains("fps") {
                (60.0 - stats.latest_value) / 60.0
            } else {
                (stats.latest_value - 16.0) / 16.0
            };
            let severity = severity.min(1.0);
            
            let description = if name.contains("fps") {
                format!("Frame rate is low at {:.2} FPS", stats.latest_value)
            } else {
                format!("Frame time is high at {:.2} ms", stats.latest_value)
            };
            
            let recommendations = vec![
                "Reduce graphics quality settings".to_string(),
                "Close background applications".to_string(),
                "Check for thermal throttling".to_string(),
                "Update graphics drivers".to_string(),
            ];
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![name.clone()],
                severity,
                description,
                recommendations,
            });
        }
        
        // Check for hardware issues
        if (name.contains("temperature") || name.contains("fan")) && 
           ((name.contains("temperature") && stats.latest_value > 80.0) || 
            (name.contains("fan") && stats.latest_value > 90.0)) {
            let insight_type = InsightType::HardwareIssue;
            
            let severity = if name.contains("temperature") {
                (stats.latest_value - 80.0) / 20.0
            } else {
                (stats.latest_value - 90.0) / 10.0
            };
            let severity = severity.min(1.0);
            
            let description = if name.contains("temperature") {
                format!("Temperature is high at {:.2}°C", stats.latest_value)
            } else {
                format!("Fan speed is high at {:.2}%", stats.latest_value)
            };
            
            let recommendations = vec![
                "Ensure proper ventilation".to_string(),
                "Clean dust from cooling system".to_string(),
                "Reduce system load".to_string(),
                "Check for thermal paste degradation".to_string(),
            ];
            
            insights.push(Insight {
                insight_type,
                related_metrics: vec![name.clone()],
                severity,
                description,
                recommendations,
            });
        }
    }
    
    // Sort insights by severity (highest first)
    insights.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(insights)
}

/// Generate recommendations based on insight type and metric.
///
/// # Arguments
///
/// * `insight_type` - Type of insight
/// * `metric_name` - Name of the metric
/// * `value` - Current value of the metric
///
/// # Returns
///
/// A vector of recommendations.
fn generate_recommendations(insight_type: InsightType, metric_name: &str, value: f64) -> Vec<String> {
    match insight_type {
        InsightType::PerformanceIssue => {
            if metric_name.contains("cpu") {
                vec![
                    "Close unnecessary applications to reduce CPU load".to_string(),
                    "Check for background processes consuming CPU".to_string(),
                    "Consider upgrading CPU if this is a recurring issue".to_string(),
                ]
            } else if metric_name.contains("memory") {
                vec![
                    "Close memory-intensive applications".to_string(),
                    "Check for memory leaks in applications".to_string(),
                    "Consider adding more RAM if this is a recurring issue".to_string(),
                ]
            } else if metric_name.contains("fps") || metric_name.contains("frame") {
                vec![
                    "Reduce graphics quality settings".to_string(),
                    "Close background applications".to_string(),
                    "Update graphics drivers".to_string(),
                    "Check for thermal throttling".to_string(),
                ]
            } else if metric_name.contains("network") || metric_name.contains("latency") {
                vec![
                    "Check network connection quality".to_string(),
                    "Reduce network-intensive applications".to_string(),
                    "Consider using a wired connection instead of WiFi".to_string(),
                    "Contact your ISP if issues persist".to_string(),
                ]
            } else {
                vec![
                    "Monitor system resources during operation".to_string(),
                    "Check for software updates".to_string(),
                    "Restart the system if issues persist".to_string(),
                ]
            }
        },
        InsightType::ResourceConstraint => {
            if metric_name.contains("cpu") {
                vec![
                    "Close unnecessary applications to reduce CPU load".to_string(),
                    "Check for background processes consuming CPU".to_string(),
                    "Consider upgrading CPU if this is a recurring issue".to_string(),
                ]
            } else if metric_name.contains("memory") {
                vec![
                    "Close memory-intensive applications".to_string(),
                    "Check for memory leaks in applications".to_string(),
                    "Consider adding more RAM if this is a recurring issue".to_string(),
                ]
            } else if metric_name.contains("disk") {
                vec![
                    "Free up disk space by removing unnecessary files".to_string(),
                    "Move large files to external storage".to_string(),
                    "Consider upgrading storage if this is a recurring issue".to_string(),
                ]
            } else {
                vec![
                    "Monitor system resources during operation".to_string(),
                    "Close unnecessary applications".to_string(),
                    "Consider hardware upgrades if issues persist".to_string(),
                ]
            }
        },
        InsightType::HardwareIssue => {
            if metric_name.contains("temperature") {
                vec![
                    "Ensure proper ventilation".to_string(),
                    "Clean dust from cooling system".to_string(),
                    "Reduce system load".to_string(),
                    "Check for thermal paste degradation".to_string(),
                ]
            } else if metric_name.contains("fan") {
                vec![
                    "Check for obstructions in cooling system".to_string(),
                    "Clean dust from fans and vents".to_string(),
                    "Check fan connections".to_string(),
                    "Replace fan if issues persist".to_string(),
                ]
            } else if metric_name.contains("battery") {
                vec![
                    "Reduce power-intensive applications".to_string(),
                    "Check battery health".to_string(),
                    "Consider replacing battery if health is poor".to_string(),
                    "Use power-saving mode".to_string(),
                ]
            } else {
                vec![
                    "Check hardware connections".to_string(),
                    "Update device drivers".to_string(),
                    "Run hardware diagnostics".to_string(),
                    "Contact support if issues persist".to_string(),
                ]
            }
        },
        InsightType::UsagePattern => {
            vec![
                "Review usage patterns for optimization opportunities".to_string(),
                "Consider adjusting settings based on usage patterns".to_string(),
                "Check for unusual activity".to_string(),
            ]
        },
        InsightType::SecurityConcern => {
            vec![
                "Check for unauthorized access".to_string(),
                "Update security software".to_string(),
                "Review security settings".to_string(),
                "Change passwords if necessary".to_string(),
            ]
        },
        InsightType::SystemHealth => {
            vec![
                "Check for software updates".to_string(),
                "Run system diagnostics".to_string(),
                "Restart the system".to_string(),
                "Contact support if issues persist".to_string(),
            ]
        },
    }
}

/// Perform linear regression on two vectors.
///
/// # Arguments
///
/// * `x` - Independent variable values
/// * `y` - Dependent variable values
///
/// # Returns
///
/// A tuple of (slope, intercept).
fn linear_regression(x: &[i64], y: &[f64]) -> Result<(f64, f64)> {
    if x.len() != y.len() {
        return Err(anyhow!("Input vectors must have the same length"));
    }
    
    if x.is_empty() {
        return Err(anyhow!("Input vectors must not be empty"));
    }
    
    let n = x.len() as f64;
    
    // Convert x to f64 for calculations
    let x_f64: Vec<f64> = x.iter().map(|&v| v as f64).collect();
    
    let sum_x: f64 = x_f64.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x_f64.iter().zip(y.iter()).map(|(&x, &y)| x * y).sum();
    let sum_xx: f64 = x_f64.iter().map(|&x| x * x).sum();
    
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;
    
    Ok((slope, intercept))
}

/// Calculate Pearson correlation coefficient between two vectors.
///
/// # Arguments
///
/// * `x` - First vector
/// * `y` - Second vector
///
/// # Returns
///
/// Pearson correlation coefficient.
fn pearson_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
    if x.len() != y.len() {
        return Err(anyhow!("Input vectors must have the same length"));
    }
    
    if x.is_empty() {
        return Err(anyhow!("Input vectors must not be empty"));
    }
    
    let n = x.len() as f64;
    
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(&x, &y)| x * y).sum();
    let sum_xx: f64 = x.iter().map(|&x| x * x).sum();
    let sum_yy: f64 = y.iter().map(|&y| y * y).sum();
    
    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_xx - sum_x * sum_x) * (n * sum_yy - sum_y * sum_y)).sqrt();
    
    if denominator == 0.0 {
        return Ok(0.0);
    }
    
    Ok(numerator / denominator)
}

/// Align two time series by timestamp.
///
/// # Arguments
///
/// * `series1` - First time series as (timestamp, value) pairs
/// * `series2` - Second time series as (timestamp, value) pairs
///
/// # Returns
///
/// A vector of aligned (value1, value2) pairs.
fn align_time_series(
    series1: &[(DateTime<Utc>, f64)],
    series2: &[(DateTime<Utc>, f64)],
) -> Vec<(f64, f64)> {
    let mut aligned = Vec::new();
    
    // Create maps of timestamp to value for both series
    let mut map1 = HashMap::new();
    let mut map2 = HashMap::new();
    
    for (t, v) in series1 {
        map1.insert(t.timestamp(), *v);
    }
    
    for (t, v) in series2 {
        map2.insert(t.timestamp(), *v);
    }
    
    // Find common timestamps
    let timestamps1: HashSet<i64> = map1.keys().cloned().collect();
    let timestamps2: HashSet<i64> = map2.keys().cloned().collect();
    
    let common_timestamps: Vec<i64> = timestamps1.intersection(&timestamps2).cloned().collect();
    
    // Create aligned pairs
    for t in common_timestamps {
        if let (Some(&v1), Some(&v2)) = (map1.get(&t), map2.get(&t)) {
            aligned.push((v1, v2));
        }
    }
    
    aligned
}

/// Generate a summary of telemetry analysis.
///
/// # Arguments
///
/// * `analysis` - Telemetry analysis results
///
/// # Returns
///
/// A summary of the analysis as a string.
pub fn generate_analysis_summary(analysis: &TelemetryAnalysis) -> String {
    let mut summary = String::new();
    
    // Add header
    summary.push_str(&format!("Telemetry Analysis Summary ({})\n", 
                             analysis.timestamp.format("%Y-%m-%d %H:%M:%S")));
    summary.push_str("=====================================\n\n");
    
    // Add insights
    summary.push_str("Key Insights:\n");
    summary.push_str("--------------\n");
    
    if analysis.insights.is_empty() {
        summary.push_str("No significant insights detected.\n");
    } else {
        for (i, insight) in analysis.insights.iter().enumerate().take(5) {
            let severity_str = match insight.severity {
                s if s > 0.7 => "Critical",
                s if s > 0.4 => "High",
                s if s > 0.2 => "Medium",
                _ => "Low",
            };
            
            summary.push_str(&format!("{}. {} ({} severity): {}\n", 
                                     i + 1, 
                                     format!("{:?}", insight.insight_type).replace("Issue", "").replace("Constraint", ""),
                                     severity_str,
                                     insight.description));
            
            // Add top recommendation
            if !insight.recommendations.is_empty() {
                summary.push_str(&format!("   Recommendation: {}\n", insight.recommendations[0]));
            }
        }
    }
    
    summary.push_str("\n");
    
    // Add anomalies
    summary.push_str("Recent Anomalies:\n");
    summary.push_str("-----------------\n");
    
    if analysis.anomalies.is_empty() {
        summary.push_str("No significant anomalies detected.\n");
    } else {
        for (i, anomaly) in analysis.anomalies.iter().enumerate().take(3) {
            summary.push_str(&format!("{}. {}\n", i + 1, anomaly.description));
        }
    }
    
    summary.push_str("\n");
    
    // Add trends
    summary.push_str("Significant Trends:\n");
    summary.push_str("-------------------\n");
    
    if analysis.trends.is_empty() {
        summary.push_str("No significant trends detected.\n");
    } else {
        for (i, trend) in analysis.trends.iter().enumerate().take(3) {
            summary.push_str(&format!("{}. {}\n", i + 1, trend.description));
        }
    }
    
    summary.push_str("\n");
    
    // Add key metrics
    summary.push_str("Key Metrics:\n");
    summary.push_str("------------\n");
    
    let key_metrics = [
        "cpu_usage_percent",
        "memory_usage_percent",
        "disk_usage_percent",
        "battery_percent",
        "temperature",
        "fps",
        "frame_time_ms",
    ];
    
    let mut found_metrics = false;
    
    for metric in key_metrics.iter() {
        for (name, stats) in &analysis.statistics {
            if name.contains(metric) {
                found_metrics = true;
                summary.push_str(&format!("{}: {:.2} (avg: {:.2}, min: {:.2}, max: {:.2})\n",
                                         name,
                                         stats.latest_value,
                                         stats.mean,
                                         stats.min,
                                         stats.max));
                
                if let Some(change) = stats.change_percent {
                    let direction = if change > 0.0 { "up" } else { "down" };
                    summary.push_str(&format!("   {} {:.2}% from previous\n", direction, change.abs()));
                }
            }
        }
    }
    
    if !found_metrics {
        summary.push_str("No key metrics available.\n");
    }
    
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_numeric_value() {
        assert_eq!(extract_numeric_value(&TelemetryValue::Integer(42)), Some(42.0));
        assert_eq!(extract_numeric_value(&TelemetryValue::Float(3.14)), Some(3.14));
        assert_eq!(extract_numeric_value(&TelemetryValue::Boolean(true)), Some(1.0));
        assert_eq!(extract_numeric_value(&TelemetryValue::Boolean(false)), Some(0.0));
        assert_eq!(extract_numeric_value(&TelemetryValue::String("test".to_string())), None);
    }
    
    #[test]
    fn test_linear_regression() {
        let x = vec![1, 2, 3, 4, 5];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let (slope, intercept) = linear_regression(&x, &y).unwrap();
        
        assert!((slope - 2.0).abs() < 0.001);
        assert!((intercept - 0.0).abs() < 0.001);
    }
    
    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let correlation = pearson_correlation(&x, &y).unwrap();
        
        assert!((correlation - 1.0).abs() < 0.001);
        
        let y_neg = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        
        let correlation_neg = pearson_correlation(&x, &y_neg).unwrap();
        
        assert!((correlation_neg + 1.0).abs() < 0.001);
    }
    
    #[test]
    fn test_align_time_series() {
        let now = Utc::now();
        let series1 = vec![
            (now, 1.0),
            (now + Duration::seconds(1), 2.0),
            (now + Duration::seconds(2), 3.0),
        ];
        
        let series2 = vec![
            (now, 10.0),
            (now + Duration::seconds(2), 30.0),
            (now + Duration::seconds(3), 40.0),
        ];
        
        let aligned = align_time_series(&series1, &series2);
        
        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0], (1.0, 10.0));
        assert_eq!(aligned[1], (3.0, 30.0));
    }
}

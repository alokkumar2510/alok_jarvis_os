use crate::database::Database;
use std::collections::HashMap;

/// Records a timing latency metric in the database
pub fn log_latency(db: &Database, metric_name: &str, value_ms: f64) {
    let _ = db.log_performance_metric(metric_name, value_ms);
}

/// Retrieves average latencies grouped by metric name
pub fn get_averages(db: &Database) -> HashMap<String, f64> {
    db.get_average_performance_metrics().unwrap_or_default()
}

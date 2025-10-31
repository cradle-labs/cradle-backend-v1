use chrono::Duration;

/// Configuration for aggregation behavior
#[derive(Clone, Debug)]
pub struct AggregatorsConfig {
    /// Whether to enable checkpoint/resume functionality
    pub enable_checkpoints: bool,
    /// Interval in seconds between checkpoint saves
    pub checkpoint_interval_secs: i64,
}

impl Default for AggregatorsConfig {
    fn default() -> Self {
        Self {
            enable_checkpoints: true,
            checkpoint_interval_secs: 300, // Save checkpoint every 5 minutes
        }
    }
}

impl AggregatorsConfig {
    pub fn new(enable_checkpoints: bool, checkpoint_interval_secs: i64) -> Self {
        Self {
            enable_checkpoints,
            checkpoint_interval_secs,
        }
    }

    pub fn checkpoint_duration(&self) -> Duration {
        Duration::seconds(self.checkpoint_interval_secs)
    }
}

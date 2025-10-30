use anyhow::{anyhow, Result};
use std::future::Future;
use colored::Colorize;
use rand::Rng;

/// Exponential backoff retry handler with jitter
#[derive(Debug, Clone)]
pub struct ExponentialBackoffRetry {
    /// Base delay in milliseconds
    base_delay_ms: u64,

    /// Maximum number of retry attempts
    max_retries: u32,

    /// Current attempt number (for calculating delay)
    current_attempt: u32,
}

impl ExponentialBackoffRetry {
    /// Create a new retry handler
    pub fn new(base_delay_ms: u64, max_retries: u32) -> Self {
        Self {
            base_delay_ms,
            max_retries,
            current_attempt: 0,
        }
    }

    /// Execute an operation with exponential backoff retry logic
    pub async fn execute<F, Fut, T>(&mut self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            self.current_attempt = attempt;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay_ms = self.calculate_delay(attempt);
                        eprintln!(
                            "{}  (attempt {}/{}) - retrying in {}ms",
                            format!("✗ Operation failed").yellow(),
                            attempt + 1,
                            self.max_retries + 1,
                            delay_ms
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms))
                            .await;
                    }
                }
            }
        }

        Err(anyhow!(
            "Operation failed after {} retries: {}",
            self.max_retries + 1,
            last_error.unwrap_or_else(|| anyhow!("unknown error"))
        ))
    }

    /// Calculate delay for current attempt using exponential backoff with jitter
    fn calculate_delay(&self, attempt: u32) -> u64 {
        let exponential_delay = self.base_delay_ms * 2_u64.pow(attempt);
        let jitter_factor = 0.1; // ±10% jitter
        let jitter_range = (exponential_delay as f64 * jitter_factor) as u64;

        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0..=jitter_range * 2) as i64 - jitter_range as i64;

        let final_delay = (exponential_delay as i64 + jitter).max(0) as u64;

        // Cap at 30 seconds to prevent excessively long waits
        final_delay.min(30_000)
    }

    /// Get the current attempt number
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }

    /// Check if we've exhausted retries
    pub fn exhausted(&self) -> bool {
        self.current_attempt > self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_successful_operation() -> Result<()> {
        let mut retry = ExponentialBackoffRetry::new(10, 3);
        let result = retry
            .execute(|| async { Ok::<i32, anyhow::Error>(42) })
            .await?;

        assert_eq!(result, 42);
        assert_eq!(retry.current_attempt(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_operation_with_retries() -> Result<()> {
        let mut retry = ExponentialBackoffRetry::new(10, 3);
        let mut attempt_count = 0;

        let result = retry
            .execute(|| async {
                attempt_count += 1;
                if attempt_count < 3 {
                    Err(anyhow!("Temporary failure"))
                } else {
                    Ok::<i32, anyhow::Error>(42)
                }
            })
            .await?;

        assert_eq!(result, 42);
        assert_eq!(attempt_count, 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_operation_exhausts_retries() {
        let mut retry = ExponentialBackoffRetry::new(10, 2);

        let result = retry
            .execute(|| async { Err::<i32, _>(anyhow!("Permanent failure")) })
            .await;

        assert!(result.is_err());
        assert!(retry.exhausted());
    }

    #[test]
    fn test_delay_calculation() {
        let retry = ExponentialBackoffRetry::new(100, 3);

        // Verify exponential growth (without jitter precision)
        let delay_0 = retry.calculate_delay(0);
        assert!(delay_0 >= 100 && delay_0 <= 120); // 100 ± 10%

        let delay_1 = retry.calculate_delay(1);
        assert!(delay_1 >= 180 && delay_1 <= 220); // 200 ± 10%

        let delay_2 = retry.calculate_delay(2);
        assert!(delay_2 >= 360 && delay_2 <= 440); // 400 ± 10%
    }
}

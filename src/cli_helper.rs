use anyhow::Result;
use std::io::{self, Write};
use colored::Colorize;
use crate::utils::app_config::AppConfig;
use crate::action_router::{ActionRouterInput, ActionRouterOutput};

/// Initialize AppConfig for CLI operations
pub fn initialize_app_config() -> Result<AppConfig> {
    AppConfig::from_env()
}

/// Execute an action through the ActionRouter
pub async fn call_action_router(input: ActionRouterInput, app_config: AppConfig) -> Result<ActionRouterOutput> {
    input.process(app_config).await
}

/// Prompt user to retry a failed operation
pub fn prompt_retry() -> Result<bool> {
    loop {
        print!("Operation failed. Retry? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("Please enter 'y' or 'n'");
                continue;
            }
        }
    }
}

/// Execute an operation with retry prompt on failure
pub async fn execute_with_retry<F, Fut, T>(
    operation: F,
    _op_name: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                eprintln!("\n{}: {}\n", "Error".red(), e);

                if prompt_retry()? {
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }
}

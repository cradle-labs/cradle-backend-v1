// Shared CLI utilities for all interactive CLIs
pub mod menu;
pub mod input;
pub mod filters;
pub mod formatting;

pub use menu::Menu;
pub use input::Input;
pub use filters::Filter;
pub use formatting::{format_table, format_json, format_record};

/// Result type for CLI operations
pub type CliResult<T> = std::result::Result<T, CliError>;

/// CLI error type
#[derive(Debug)]
pub enum CliError {
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    UserCancelled,
    IoError(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            CliError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CliError::NotFound(msg) => write!(f, "Not found: {}", msg),
            CliError::UserCancelled => write!(f, "Operation cancelled by user"),
            CliError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError(err.to_string())
    }
}

/// Confirm a dangerous operation
pub fn confirm(message: &str) -> CliResult<bool> {
    use dialoguer::Confirm;
    Confirm::new()
        .with_prompt(message)
        .interact()
        .map_err(|e| CliError::IoError(e.to_string()))
}

/// Print a success message
pub fn print_success(message: &str) {
    use colored::Colorize;
    eprintln!("{}", format!("✓ {}", message).green());
}

/// Print an info message
pub fn print_info(message: &str) {
    use colored::Colorize;
    eprintln!("{}", format!("ℹ {}", message).bright_cyan());
}

/// Print a warning message
pub fn print_warning(message: &str) {
    use colored::Colorize;
    eprintln!("{}", format!("⚠ {}", message).yellow());
}

/// Print an error message
pub fn print_error(message: &str) {
    use colored::Colorize;
    eprintln!("{}", format!("✗ {}", message).red());
}

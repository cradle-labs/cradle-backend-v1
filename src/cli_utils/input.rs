use crate::cli_utils::CliResult;
use dialoguer::Input as DialoguerInput;
use uuid::Uuid;

/// Input utilities
pub struct Input;

impl Input {
    /// Get a UUID from user input
    pub fn get_uuid(prompt: &str) -> CliResult<Uuid> {
        let input: String = DialoguerInput::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

        Uuid::parse_str(&input)
            .map_err(|_| crate::cli_utils::CliError::ValidationError(
                "Invalid UUID format".to_string()
            ))
    }

    /// Get a string from user input
    pub fn get_string(prompt: &str) -> CliResult<String> {
        DialoguerInput::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))
    }

    /// Get an optional string
    pub fn get_optional_string(prompt: &str) -> CliResult<Option<String>> {
        let input: String = DialoguerInput::new()
            .with_prompt(&format!("{} (optional)", prompt))
            .allow_empty(true)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

        Ok(if input.is_empty() { None } else { Some(input) })
    }

    /// Get an integer
    pub fn get_i64(prompt: &str) -> CliResult<i64> {
        let input: String = DialoguerInput::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

        input.parse::<i64>()
            .map_err(|_| crate::cli_utils::CliError::ValidationError(
                "Invalid number format".to_string()
            ))
    }

    /// Get a decimal number
    pub fn get_decimal(prompt: &str) -> CliResult<bigdecimal::BigDecimal> {
        use std::str::FromStr;
        let input: String = DialoguerInput::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

        bigdecimal::BigDecimal::from_str(&input)
            .map_err(|_| crate::cli_utils::CliError::ValidationError(
                "Invalid decimal format".to_string()
            ))
    }

    /// Get a boolean choice
    pub fn get_bool(prompt: &str) -> CliResult<bool> {
        use dialoguer::Confirm;
        Confirm::new()
            .with_prompt(prompt)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))
    }

    /// Select from a list
    pub fn select_from_list(prompt: &str, items: Vec<&str>) -> CliResult<usize> {
        use dialoguer::Select;
        Select::new()
            .with_prompt(prompt)
            .items(&items)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))
    }

    /// Select a string from a list
    pub fn select_string(prompt: &str, items: Vec<String>) -> CliResult<String> {
        use dialoguer::Select;
        let item_refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        let idx = Select::new()
            .with_prompt(prompt)
            .items(&item_refs)
            .interact()
            .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;
        Ok(items[idx].clone())
    }
}

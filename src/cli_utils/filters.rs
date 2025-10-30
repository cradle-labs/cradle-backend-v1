use crate::cli_utils::{CliResult, Input};
use std::collections::HashMap;

/// Simple filter builder for queries
pub struct Filter {
    filters: HashMap<String, String>,
}

impl Filter {
    /// Create a new filter
    pub fn new() -> Self {
        Self {
            filters: HashMap::new(),
        }
    }

    /// Add a filter condition
    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.filters.insert(key.to_string(), value.to_string());
        self
    }

    /// Get a filter value
    pub fn get(&self, key: &str) -> Option<&String> {
        self.filters.get(key)
    }

    /// Get all filters
    pub fn all(&self) -> &HashMap<String, String> {
        &self.filters
    }

    /// Check if a filter exists
    pub fn has(&self, key: &str) -> bool {
        self.filters.contains_key(key)
    }

    /// Clear all filters
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Get as SQL WHERE clause components (simplified)
    pub fn to_sql_parts(&self) -> Vec<String> {
        self.filters
            .iter()
            .map(|(k, v)| format!("{} = '{}'", k, v))
            .collect()
    }

    /// Count active filters
    pub fn count(&self) -> usize {
        self.filters.len()
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

/// Status filter helper
pub fn status_filter_options() -> Vec<&'static str> {
    vec!["All", "Active", "Inactive", "Pending", "Cancelled"]
}

/// Type filter helper
pub fn type_filter_options() -> Vec<&'static str> {
    vec!["All", "Individual", "Business", "Service"]
}

/// Prompt for status filter
pub fn prompt_status_filter() -> CliResult<Option<String>> {
    use dialoguer::Select;

    let options = status_filter_options();
    let selected = Select::new()
        .with_prompt("Filter by status")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

    Ok(match selected {
        0 => None, // All
        1 => Some("active".to_string()),
        2 => Some("inactive".to_string()),
        3 => Some("pending".to_string()),
        4 => Some("cancelled".to_string()),
        _ => None,
    })
}

/// Prompt for type filter
pub fn prompt_type_filter() -> CliResult<Option<String>> {
    use dialoguer::Select;

    let options = type_filter_options();
    let selected = Select::new()
        .with_prompt("Filter by type")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| crate::cli_utils::CliError::IoError(e.to_string()))?;

    Ok(match selected {
        0 => None, // All
        1 => Some("individual".to_string()),
        2 => Some("business".to_string()),
        3 => Some("service".to_string()),
        _ => None,
    })
}

/// Prompt for date range filter
pub fn prompt_date_range() -> CliResult<(String, String)> {
    use chrono::Local;

    let start = Input::get_string("Start date (YYYY-MM-DD) or blank for 30 days ago")
        .unwrap_or_else(|_| {
            let thirty_days_ago = Local::now() - chrono::Duration::days(30);
            thirty_days_ago.format("%Y-%m-%d").to_string()
        });

    let end = Input::get_string("End date (YYYY-MM-DD) or blank for today")
        .unwrap_or_else(|_| Local::now().format("%Y-%m-%d").to_string());

    Ok((start, end))
}

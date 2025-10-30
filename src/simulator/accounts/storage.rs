use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use super::models::GeneratedBatch;

/// Save a batch of generated accounts to a JSON file
pub fn save_batch_to_json(batch: &GeneratedBatch, path: &Path) -> Result<()> {
    let json_string = serde_json::to_string_pretty(batch)
        .map_err(|e| anyhow!("Failed to serialize batch to JSON: {}", e))?;

    fs::write(path, json_string)
        .map_err(|e| anyhow!("Failed to write JSON file to {}: {}", path.display(), e))?;

    Ok(())
}

/// Load a batch of generated accounts from a JSON file
pub fn load_batch_from_json(path: &Path) -> Result<GeneratedBatch> {
    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    let json_string = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read JSON file from {}: {}", path.display(), e))?;

    let batch: GeneratedBatch = serde_json::from_str(&json_string)
        .map_err(|e| anyhow!("Failed to deserialize JSON: {}", e))?;

    Ok(batch)
}

/// Append new accounts to an existing batch file (creates file if it doesn't exist)
pub fn append_to_existing_batch(new_batch: &GeneratedBatch, path: &Path) -> Result<()> {
    let mut batch = if path.exists() {
        load_batch_from_json(path)?
    } else {
        // Create a new batch with the same config
        GeneratedBatch::new(new_batch.config.clone())
    };

    // Extend accounts and update stats
    batch.accounts.extend(new_batch.accounts.clone());
    batch.stats.total_requested += new_batch.stats.total_requested;
    batch.stats.successfully_created += new_batch.stats.successfully_created;
    batch.stats.failed_count += new_batch.stats.failed_count;
    batch.stats.total_associations += new_batch.stats.total_associations;
    batch.stats.successful_associations += new_batch.stats.successful_associations;
    batch.stats.total_kyc_grants += new_batch.stats.total_kyc_grants;
    batch.stats.successful_kyc_grants += new_batch.stats.successful_kyc_grants;

    batch.mark_completed();

    save_batch_to_json(&batch, path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_and_load_batch() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        let config = crate::simulator::accounts::GeneratorConfig::new();
        let mut batch = GeneratedBatch::new(config);
        batch.stats.total_requested = 5;
        batch.stats.successfully_created = 5;

        save_batch_to_json(&batch, path)?;

        let loaded_batch = load_batch_from_json(path)?;
        assert_eq!(loaded_batch.stats.total_requested, 5);
        assert_eq!(loaded_batch.stats.successfully_created, 5);

        Ok(())
    }
}

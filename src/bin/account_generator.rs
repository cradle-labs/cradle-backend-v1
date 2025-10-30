use anyhow::{anyhow, Result};
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

use cradle_back_end::simulator::accounts::{
    AccountGenerator, GeneratorConfig, save_batch_to_json,
};
use cradle_back_end::accounts::db_types::{CradleAccountStatus, CradleAccountType};
use cradle_back_end::cli_helper::initialize_app_config;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!(
        "{}",
        "╔═══════════════════════════════════════════════════════╗".bright_cyan()
    );
    eprintln!(
        "{}",
        "║     Cradle Account Generator (Batch Simulator)        ║".bright_cyan()
    );
    eprintln!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".bright_cyan()
    );
    eprintln!();

    // Parse command line arguments
    let args = std::env::args().collect::<Vec<_>>();
    let config = parse_args(&args)?;

    // Initialize app config
    eprint!("Initializing app config... ");
    std::io::stderr().flush().ok();

    let app_config = match initialize_app_config() {
        Ok(cfg) => {
            eprintln!("{}", "✓ Ready".green());
            cfg
        }
        Err(e) => {
            eprintln!("{}", "✗ Failed".red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    eprintln!();

    // Create generator and run batch
    let generator = AccountGenerator::new(config.clone(), app_config);

    match generator.generate_batch().await {
        Ok(batch) => {
            // Save to JSON
            eprint!("Saving batch to JSON... ");
            std::io::stderr().flush().ok();

            match save_batch_to_json(&batch, &config.output_file) {
                Ok(_) => {
                    eprintln!("{}", "✓ Done".green());
                    eprintln!(
                        "  {} Saved to: {}",
                        "ℹ".bright_blue(),
                        config.output_file.display()
                    );
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{}", "✗ Failed".red());
                    Err(e)
                }
            }
        }
        Err(e) => {
            eprintln!("{}", "✗ Generation failed".red());
            Err(e)
        }
    }
}

/// Parse command line arguments into GeneratorConfig
fn parse_args(args: &[String]) -> Result<GeneratorConfig> {
    let mut config = GeneratorConfig::default();
    let mut i = 1; // Skip program name

    while i < args.len() {
        match args[i].as_str() {
            "--batch-size" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --batch-size"));
                }
                config.batch_size = args[i].parse::<u32>()?;
            }
            "--account-type" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --account-type"));
                }
                config.account_type = match args[i].to_lowercase().as_str() {
                    "retail" => CradleAccountType::Retail,
                    "institutional" => CradleAccountType::Institutional,
                    other => {
                        return Err(anyhow!(
                            "Invalid account type: {}. Use 'retail' or 'institutional'",
                            other
                        ))
                    }
                };
            }
            "--assets" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --assets"));
                }
                let assets: Result<Vec<Uuid>> = args[i]
                    .split(',')
                    .map(|s| {
                        Uuid::parse_str(s.trim())
                            .map_err(|e| anyhow!("Invalid UUID: {}", e))
                    })
                    .collect();
                config.assets_to_associate = assets?;
            }
            "--apply-kyc" => {
                config.apply_kyc = true;
            }
            "--no-kyc" => {
                config.apply_kyc = false;
            }
            "--output" | "--output-file" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --output"));
                }
                config.output_file = PathBuf::from(&args[i]);
            }
            "--initial-status" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --initial-status"));
                }
                config.initial_status = match args[i].to_lowercase().as_str() {
                    "unverified" => CradleAccountStatus::UnVerified,
                    "verified" => CradleAccountStatus::Verified,
                    "suspended" => CradleAccountStatus::Suspended,
                    "closed" => CradleAccountStatus::Closed,
                    other => {
                        return Err(anyhow!(
                            "Invalid status: {}. Use 'unverified', 'verified', 'suspended', or 'closed'",
                            other
                        ))
                    }
                };
            }
            "--retry-limit" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --retry-limit"));
                }
                config.retry_limit = args[i].parse::<u32>()?;
            }
            "--retry-delay" => {
                i += 1;
                if i >= args.len() {
                    return Err(anyhow!("Missing value for --retry-delay"));
                }
                config.retry_delay_ms = args[i].parse::<u64>()?;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg => {
                return Err(anyhow!("Unknown argument: {}", arg));
            }
        }

        i += 1;
    }

    Ok(config)
}

fn print_help() {
    println!(
        "{}",
        r#"
Cradle Account Generator - Batch Account Creation Simulator

USAGE:
    account_generator [OPTIONS]

OPTIONS:
    --batch-size <COUNT>
        Number of accounts to generate (default: 10)

    --account-type <TYPE>
        Account type: 'retail' or 'institutional' (default: retail)

    --assets <UUID1>,<UUID2>,...
        Comma-separated list of asset UUIDs to associate with each account

    --apply-kyc
        Enable KYC granting for associated assets (default: false)

    --no-kyc
        Disable KYC granting (explicitly set to false)

    --output <FILE>
        Output JSON file path (default: simulated_accounts.json)

    --initial-status <STATUS>
        Initial account status: 'unverified', 'verified', 'suspended', or 'closed'
        (default: unverified)

    --retry-limit <COUNT>
        Maximum retry attempts per operation (default: 3)

    --retry-delay <MILLISECONDS>
        Base delay for exponential backoff in milliseconds (default: 500)

    --help, -h
        Show this help message

EXAMPLES:
    # Create 10 retail accounts
    account_generator --batch-size 10

    # Create 50 accounts with asset association
    account_generator --batch-size 50 --assets a1b2c3d4-e5f6-4a5b-6c7d-8e9f0a1b2c3d

    # Create accounts with assets and KYC
    account_generator \
        --batch-size 20 \
        --account-type retail \
        --assets uuid1,uuid2,uuid3 \
        --apply-kyc

    # Create institutional accounts with custom output
    account_generator \
        --batch-size 5 \
        --account-type institutional \
        --output institutional_accounts.json
"#
    );
}

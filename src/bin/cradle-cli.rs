use anyhow::Result;
use colored::Colorize;
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    loop {
        print_banner();

        let modules = vec![
            ("Accounts", "accounts-cli"),
            ("Asset Book", "asset-book-cli"),
            ("Lending Pool", "lending-pool-cli"),
            ("Markets", "market-cli"),
            ("Order Book", "order-book-cli"),
            ("Market Time Series", "market-time-series-cli"),
            ("Timeseries Aggregator", "timeseries-aggregator"),
            ("Exit", ""),
        ];

        println!("{}", "╭─────────────────────────────────────╮".bright_cyan());
        println!("{}", "│  Select a module to manage:         │".bright_cyan());
        println!("{}", "╰─────────────────────────────────────╯".bright_cyan());
        println!();

        for (i, (name, _)) in modules.iter().enumerate() {
            println!("  {}. {}", i, name);
        }
        println!();

        let selection = get_selection(modules.len() as i32)?;

        if selection as usize >= modules.len() {
            println!("{}", "Invalid selection".red());
            continue;
        }

        let (name, binary) = modules[selection as usize];

        if binary.is_empty() {
            eprintln!("{}", "Goodbye!".bright_cyan());
            break;
        }

        println!();
        eprintln!("{}", format!("Launching {}...", name).bright_cyan());
        eprintln!();

        // Launch the selected CLI binary
        let status = Command::new(format!("cargo"))
            .args(&["run", "--bin", binary, "--release"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()?;

        if !status.success() {
            eprintln!("{}", "Command failed with non-zero exit code".red());
        }

        eprintln!();
    }

    Ok(())
}

fn print_banner() {
    eprintln!();
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║                                                       ║".bright_cyan());
    eprintln!("{}", "║         🏗️  Cradle Platform Management CLI            ║".bright_cyan());
    eprintln!("{}", "║                                                       ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    eprintln!();
}

fn get_selection(max: i32) -> Result<i32> {
    use std::io::{self, Write};

    print!("Enter your choice (0-{}): ", max - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selection = input.trim().parse::<i32>()?;
    Ok(selection)
}

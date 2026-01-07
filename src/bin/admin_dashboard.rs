use anyhow::Result;
use colored::Colorize;
use cradle_back_end::cli_helper::initialize_app_config;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// Include the admin_ui module directly
#[path = "../admin_ui/mod.rs"]
mod admin_ui;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("{}", "╔═══════════════════════════════════════════════════════╗".bright_cyan());
    eprintln!("{}", "║         Cradle Admin Dashboard Server                 ║".bright_cyan());
    eprintln!("{}", "╚═══════════════════════════════════════════════════════╝".bright_cyan());
    eprintln!();

    eprint!("Initializing app config... ");
    let app_config = match initialize_app_config() {
        Ok(config) => {
            eprintln!("{}", "✓ Ready".green());
            config
        }
        Err(e) => {
            eprintln!("{}", "✗ Failed".red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    let router = admin_ui::router(app_config);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    eprintln!("Listening on {}", addr);
    eprintln!("Open http://localhost:3000 in your browser");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

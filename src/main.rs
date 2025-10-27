pub mod accounts;
pub mod api;
pub mod utils;
pub mod schema;
mod action_router;
mod asset_book;
mod market;
mod market_time_series;
mod order_book;
mod lending_pool;

use axum::{
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::env;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use api::{
    config::ApiConfig,
    error::ApiError,
    handlers::{
        accounts::*, assets::*, health, lending_pools::*, markets::*, mutation::*,
        orders::*, time_series::*,
    },
    middleware::auth::validate_auth,
};
use utils::app_config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string())
                .as_str(),
        )
        .init();

    // Load API configuration
    let api_config = ApiConfig::from_env();
    tracing::info!("API configuration loaded successfully");

    // Load AppConfig (database and wallet)
    let app_config = AppConfig::from_env()?;
    tracing::info!("Application configuration loaded successfully");

    // Create authentication middleware that captures the secret key
    let secret_key = api_config.secret_key.clone();

    // Custom auth middleware
    let auth_layer = middleware::from_fn(move |req: axum::extract::Request, next: Next| {
        let secret = secret_key.clone();
        async move {
            // Skip auth for /health endpoint
            let path = req.uri().path();
            if path == "/health" {
                return Ok::<Response, ApiError>(next.run(req).await.into_response());
            }

            validate_auth(req.headers(), &secret).await?;
            Ok::<Response, ApiError>(next.run(req).await.into_response())
        }
    });

    // Build router with all routes
    let router = Router::new()
        // Health check - public endpoint
        .route("/health", get(health::health))

        // Mutation endpoint
        .route("/process", post(process_mutation))

        // Accounts endpoints
        .route("/accounts/:id", get(get_account_by_id))
        .route("/accounts/linked/:linked_id", get(get_account_by_linked_id))
        .route("/accounts/:account_id/wallets", get(get_account_wallets))
        .route("/wallets/:id", get(get_wallet_by_id))
        .route("/wallets/account/:account_id", get(get_wallet_by_account_id))

        // Assets endpoints
        .route("/assets/:id", get(get_asset_by_id))
        .route("/assets/token/:token", get(get_asset_by_token))
        .route("/assets/manager/:manager", get(get_asset_by_manager))

        // Markets endpoints
        .route("/markets/:id", get(get_market_by_id))
        .route("/markets", get(get_markets))

        // Orders endpoints
        .route("/orders/:id", get(get_order_by_id))
        .route("/orders", get(get_orders))

        // Time series endpoints
        .route("/time-series/history", get(get_time_series_history))

        // Lending pools endpoints
        .route("/pools/:id", get(get_pool_by_id))
        .route("/pools/name/:name", get(get_pool_by_name))
        .route("/pools/address/:address", get(get_pool_by_address))
        .route("/pools/:id/snapshot", get(get_pool_snapshot))

        // Add middleware layers before state binding
        .layer(TraceLayer::new_for_http())
        .layer(auth_layer)

        // Shared state - applied after middleware
        .with_state(app_config);

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Starting Cradle API server on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}

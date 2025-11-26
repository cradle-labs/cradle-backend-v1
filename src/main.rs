pub mod accounts;
mod accounts_ledger;
mod action_router;
mod aggregators;
pub mod api;
mod asset_book;
mod lending_pool;
mod listing;
mod market;
mod market_time_series;
mod order_book;
pub mod schema;
mod sockets;
pub mod utils;

use axum::{
    Router,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dotenvy::dotenv;
use socketioxide::SocketIo;
use std::env;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

use crate::{
    api::handlers::{
        faucet_request::airdrop_request,
        listings::{get_listing_by_id, get_listings},
    },
    sockets::on_connect,
};
use api::{
    config::ApiConfig,
    error::ApiError,
    handlers::{
        accounts::*, assets::*, health, lending_pools::*, markets::*, mutation::*, orders::*,
        time_series::*,
    },
    middleware::auth::validate_auth,
};
use utils::app_config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenv();
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string())
                .as_str(),
        )
        .init();

    let (socket_layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);

    // Load API configuration
    let api_config = ApiConfig::from_env();

    tracing::info!("API configuration loaded successfully");

    // Load AppConfig (database and wallet)
    let mut app_config = AppConfig::from_env()?;
    app_config.set_io(io);
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
        .route(
            "/wallets/account/:account_id",
            get(get_wallet_by_account_id),
        )
        .route("/balances/:account_id", get(api_get_account_balances))
        .route("/balance/:wallet_id/:asset_id", get(get_asset_balance))
        // Assets endpoints
        .route("/assets/:id", get(get_asset_by_id))
        .route("/assets/token/:token", get(get_asset_by_token))
        .route("/assets/manager/:manager", get(get_asset_by_manager))
        .route("/assets", get(get_assets))
        // Markets endpoints
        .route("/markets/:id", get(get_market_by_id))
        .route("/markets", get(get_markets))
        // Orders endpoints
        .route("/orders/:id", get(get_order_by_id))
        .route("/orders", get(get_orders))
        // Time series endpoints
        .route("/time-series/history", get(get_time_series_history))
        // faucet request
        .route("/faucet", post(airdrop_request))
        // listings
        .route("/listings", get(get_listings))
        .route("/listings/:listing_id", get(get_listing_by_id))
        // Lending Pool
        .route("/pools", get(get_pools))
        .route("/pools/:id", get(get_pool))
        .route("/loans/:wallet", get(get_loans_handler))
        .route("/pool-stats/:id", get(get_pool_stats_handler))
        .route("/loan-position/id", get(get_pool_borrow_positions))
        .route(
            "/pools/deposit/:pool_id/:wallet_id",
            get(get_pool_deposit_handler),
        )
        .route(
            "/loans/repayments/:loan_id",
            get(get_loan_repayments_handler),
        )
        .route("/loans/:loan_id", get(get_repaid_handler))
        // Add middleware layers before state binding
        .layer(TraceLayer::new_for_http())
        .layer(auth_layer)
        .layer(socket_layer)
        .layer(CorsLayer::permissive()) // TODO: temp redo correctly once we have a domain
        // Shared state - applied after middleware
        .with_state(app_config);

    // Get port from environment or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "6969".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Starting Cradle API server on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}

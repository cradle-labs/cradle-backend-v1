use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use bigdecimal::{BigDecimal, ToPrimitive};
use std::str::FromStr;
use chrono::{Duration, Local, NaiveDateTime};

use cradle_back_end::utils::app_config::AppConfig;
use cradle_back_end::accounts::db_types::{CradleWalletAccountRecord, CreateCradleAccount, CradleAccountType, CradleAccountStatus};
use cradle_back_end::market::processor_enums::MarketProcessorInput;
use cradle_back_end::market::db_types::{MarketRecord, MarketType, MarketStatus, MarketRegulation, CreateMarket};
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::cli_helper::call_action_router;

// Asset book
use cradle_back_end::asset_book::db_types::AssetType;
use cradle_back_end::asset_book::processor_enums::{
    AssetBookProcessorInput, CreateNewAssetInputArgs, CreateExistingAssetInputArgs,
};

// Market time series
use cradle_back_end::market_time_series::db_types::{CreateMarketTimeSeriesRecord, TimeSeriesInterval, DataProviderType};
use cradle_back_end::market_time_series::processor_enum::MarketTimeSeriesProcessorInput;
use cradle_back_end::order_book::db_types::OrderBookTradeRecord;

// Ops for Faucet/OnRamp
use cradle_back_end::ramper::{Ramper, OnRampRequest};
use cradle_back_end::accounts::operations::{associate_token, kyc_token, update_asset_book_record, AssetRecordAction};
use cradle_back_end::accounts::processor_enums::{AssociateTokenToWalletInputArgs, GrantKYCInputArgs, AccountsProcessorInput};
use contract_integrator::utils::functions::cradle_account::{AssociateTokenArgs, CradleAccountFunctionInput, CradleAccountFunctionOutput};
use cradle_back_end::asset_book::operations::{get_asset, get_wallet, mint_asset};
use contract_integrator::utils::functions::{
    ContractCallInput, ContractCallOutput,
    asset_manager::{AirdropArgs, AssetManagerFunctionInput, AssetManagerFunctionOutput},
    commons::{self, ContractFunctionProcessor, get_account_balances},
};

// Lending pool ops
use cradle_back_end::lending_pool::db_types::{LendingPoolRecord, LoanRecord};
use cradle_back_end::lending_pool::processor_enums::{
    LendingPoolFunctionsInput, SupplyLiquidityInputArgs, WithdrawLiquidityInputArgs,
    TakeLoanInputArgs, RepayLoanInputArgs
};
use cradle_back_end::lending_pool::operations::{get_pool_stats, get_pool_deposit_position, get_loan_position};

// Listing ops
use cradle_back_end::listing::db_types::{CompanyRow, CradleNativeListingRow, ListingStatus};
use cradle_back_end::listing::processor_enums::CradleNativeListingFunctionsInput;
use cradle_back_end::listing::operations::{
    AssetDetails, GetPurchaseFeeInputArgs, CreateCompanyInputArgs,
    CreateListingInputArgs, PurchaseListingAssetInputArgs,
    ReturnAssetListingInputArgs, WithdrawToBeneficiaryInputArgsBody
};

// Oracle ops
use cradle_back_end::lending_pool::oracle::publish_price;
use cradle_back_end::lending_pool::operations::get_pool;

mod templates;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

pub fn router(config: AppConfig) -> Router {
    let state = AppState {
        config: Arc::new(config),
    };

    Router::new()
        .route("/", get(index_handler))
        .route("/ui/accounts", get(get_accounts_handler))
        .route("/ui/dashboard/:account_id", get(dashboard_handler))
        // Tabs
        .route("/ui/tabs/markets", get(markets_tab_handler))
        .route("/ui/tabs/onramp", get(on_ramp_tab_handler))
        .route("/ui/tabs/faucet", get(faucet_tab_handler))
        .route("/ui/tabs/lending", get(lending_tab_handler))
        // Actions
        .route("/ui/market_detail", get(market_detail_handler))
        .route("/ui/order", post(place_order_handler))
        .route("/ui/on_ramp", post(on_ramp_handler))
        .route("/ui/faucet", post(faucet_handler))
        // Lending actions
        .route("/ui/lending/supply_form", get(supply_form_handler))
        .route("/ui/lending/borrow_form", get(borrow_form_handler))
        .route("/ui/lending/withdraw_form", get(withdraw_form_handler))
        .route("/ui/lending/repay_form", get(repay_form_handler))
        .route("/ui/lending/supply", post(supply_liquidity_handler))
        .route("/ui/lending/withdraw", post(withdraw_liquidity_handler))
        .route("/ui/lending/borrow", post(borrow_handler))
        .route("/ui/lending/repay", post(repay_handler))
        .route("/ui/lending/pool_stats", get(pool_stats_handler))
        .route("/ui/lending/user_positions", get(user_positions_handler))
        // Listing tab and forms
        .route("/ui/tabs/listings", get(listings_tab_handler))
        .route("/ui/listings/create_company_form", get(create_company_form_handler))
        .route("/ui/listings/create_listing_form", get(create_listing_form_handler))
        .route("/ui/listings/purchase_form", get(purchase_form_handler))
        .route("/ui/listings/return_form", get(return_form_handler))
        .route("/ui/listings/withdraw_form", get(withdraw_listing_form_handler))
        // Listing actions
        .route("/ui/listings/create_company", post(create_company_handler))
        .route("/ui/listings/create_listing", post(create_listing_handler))
        .route("/ui/listings/purchase", post(purchase_listing_handler))
        .route("/ui/listings/return", post(return_listing_handler))
        .route("/ui/listings/withdraw", post(withdraw_listing_handler))
        .route("/ui/listings/stats", get(listing_stats_handler))
        // Oracle
        .route("/ui/tabs/oracle", get(oracle_tab_handler))
        .route("/ui/oracle/set_price", post(set_oracle_price_handler))
        // Global Admin Tools (no account selection required)
        .route("/ui/admin", get(admin_tools_handler))
        .route("/ui/admin/tabs/assets", get(admin_assets_tab_handler))
        .route("/ui/admin/tabs/markets", get(admin_markets_tab_handler))
        .route("/ui/admin/tabs/aggregator", get(admin_aggregator_tab_handler))
        .route("/ui/admin/tabs/accounts", get(admin_accounts_tab_handler))
        // Asset management
        .route("/ui/admin/assets/create_new_form", get(create_new_asset_form_handler))
        .route("/ui/admin/assets/create_existing_form", get(create_existing_asset_form_handler))
        .route("/ui/admin/assets/create_new", post(create_new_asset_handler))
        .route("/ui/admin/assets/create_existing", post(create_existing_asset_handler))
        // Market management
        .route("/ui/admin/markets/create_form", get(create_market_form_handler))
        .route("/ui/admin/markets/create", post(create_market_handler))
        .route("/ui/admin/markets/update_status", post(update_market_status_handler))
        // Aggregator
        .route("/ui/admin/aggregator/run", post(run_aggregator_handler))
        .route("/ui/admin/aggregator/run_batch", post(run_batch_aggregator_handler))
        .route("/ui/admin/aggregator/markets", get(aggregator_markets_handler))
        // Account Management (Associations & KYC)
        .route("/ui/admin/accounts/associate", post(admin_associate_token_handler))
        .route("/ui/admin/accounts/kyc", post(admin_grant_kyc_handler))
        .route("/ui/admin/accounts/associate_and_kyc", post(admin_associate_and_kyc_handler))
        .with_state(state)
}

async fn index_handler() -> Html<String> {
    Html(templates::index_page())
}

async fn get_accounts_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    // Using fully qualified paths to avoid clashes
    use cradle_back_end::schema::cradlewalletaccounts::dsl as wa_dsl;
    use cradle_back_end::schema::cradleaccounts::dsl as ca_dsl;

    let pool = state.config.pool.clone();
    
    let accounts_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        // Join cradlewalletaccounts (wa) with cradleaccounts (ca)
        // Filter where ca.account_type == Retail
        wa_dsl::cradlewalletaccounts
            .inner_join(ca_dsl::cradleaccounts.on(wa_dsl::cradle_account_id.eq(ca_dsl::id)))
            // .filter(ca_dsl::account_type.eq(CradleAccountType::Retail))
            .select(wa_dsl::cradlewalletaccounts::all_columns())
            .load::<CradleWalletAccountRecord>(&mut conn)
    }).await.unwrap();

    match accounts_result {
        Ok(accounts) => Html(templates::account_list(accounts)),
        Err(e) => Html(format!("<div class='text-red-500'>Failed to load accounts: {}</div>", e)),
    }
}

async fn dashboard_handler(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::cradlewalletaccounts::dsl as wa_dsl;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::schema::accountassetbook::dsl as aab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    use cradle_back_end::accounts_ledger::sql_queries::get_deductions;
    use contract_integrator::hedera::TokenId;
    use bigdecimal::ToPrimitive;
    
    let pool = state.config.pool.clone();
    let acc_id_copy = account_id;
    let pool_copy = pool.clone();

    // Spawn blocking task for DB operations
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool_copy.get().expect("Failed to get db connection");

        // 1. Get Wallet
        let wallet = wa_dsl::cradlewalletaccounts
            .find(acc_id_copy)
            .first::<CradleWalletAccountRecord>(&mut conn)
            .ok();

        // 2. Get Associated Assets (Joined with AssetBook)
        let assets = if let Some(w) = &wallet {
            aab_dsl::accountassetbook
                .inner_join(ab_dsl::asset_book)
                .filter(aab_dsl::account_id.eq(w.id))
                .filter(aab_dsl::associated.eq(true))
                .select(ab_dsl::asset_book::all_columns())
                .load::<AssetBookRecord>(&mut conn)
                .ok()
        } else {
            None
        };

        (wallet, assets)
    }).await.unwrap();

    let (wallet_opt, assets_opt) = result;
    
    let mut balances = Vec::new();

    if let Some(wallet) = wallet_opt {
        eprintln!("[DEBUG] Fetching balances for wallet: {} (contract_id: {})", wallet.id, wallet.contract_id);
        
        // Fetch on-chain balances ONCE using contract_id (following get_asset_balance pattern)
        match get_account_balances(&state.config.wallet.client, &wallet.contract_id).await {
            Ok(balance_data) => {
                 // HBAR
                 if let Some(hbar_val) = balance_data.hbars.get_value().to_i64() {
                     balances.push(templates::Balance {
                         token: "HBAR".to_string(),
                         amount: hbar_val.to_string()
                     });
                 }
                 
                 // Tokens (Filter by what we found in DB)
                 if let Some(assets) = assets_opt {
                     let pool_for_deductions = state.config.pool.clone();
                     
                     for asset in assets {
                         // Following get_asset_balance pattern exactly
                         match TokenId::from_solidity_address(&asset.token) {
                             Ok(token_id) => {
                                 let raw_balance = *balance_data.tokens.get(&token_id).unwrap_or(&0);
                                 
                                 // Get deductions (blocking operation)
                                 let pool_clone = pool_for_deductions.clone();
                                 let wallet_address = wallet.address.clone();
                                 let asset_id = asset.id;
                                 
                                 let deduction_result = tokio::task::spawn_blocking(move || {
                                     let mut conn = pool_clone.get().ok()?;
                                     get_deductions(&mut conn, wallet_address, asset_id).ok()
                                 }).await.unwrap();
                                 
                                 let deductions_u64 = if let Some(deductions) = deduction_result {
                                     deductions.total.to_u64().unwrap_or(0)
                                 } else {
                                     eprintln!("[WARN] Failed to get deductions for asset {}", asset.symbol);
                                     0
                                 };
                                 
                                 let net = raw_balance.saturating_sub(deductions_u64);
                                 
                                 balances.push(templates::Balance {
                                     token: asset.symbol.clone(),
                                     amount: net.to_string() 
                                 });
                             },
                             Err(e) => {
                                 eprintln!("[ERROR] Failed to parse token ID for asset {}: {:?}", asset.symbol, e);
                                 balances.push(templates::Balance {
                                     token: asset.symbol.clone(),
                                     amount: "Parse Error".to_string() 
                                 });
                             }
                         }
                     }
                 }
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to fetch account balances: {:?}", e);
                balances.push(templates::Balance { token: "Status".to_string(), amount: "Network Error".to_string() });
            }
        }
    } else {
         balances.push(templates::Balance { token: "Error".to_string(), amount: "Wallet Not Found".to_string() });
    }

    Html(templates::dashboard(account_id, balances))
}

// --- TAB HANDLERS ---

#[derive(Deserialize)]
struct TabQuery {
    account_id: Uuid,
}

async fn markets_tab_handler(State(state): State<AppState>, Query(q): Query<TabQuery>) -> Html<String> {
    use cradle_back_end::schema::markets::dsl::*;
    use cradle_back_end::market::db_types::MarketRecord;
    use diesel::prelude::*;
    
    let pool = state.config.pool.clone();
    let markets_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        markets.load::<MarketRecord>(&mut conn)
    }).await.unwrap();
    
    let markets_list = markets_result.unwrap_or_default();
    Html(templates::markets_tab(q.account_id, markets_list))
}

async fn on_ramp_tab_handler(Query(q): Query<TabQuery>) -> Html<String> {
    Html(templates::on_ramp_tab(q.account_id))
}

async fn faucet_tab_handler(State(state): State<AppState>, Query(q): Query<TabQuery>) -> Html<String> {
    use cradle_back_end::schema::asset_book::dsl::*;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    use diesel::prelude::*;

    let pool = state.config.pool.clone();
    let assets_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        asset_book.load::<AssetBookRecord>(&mut conn)
    }).await.unwrap();

    let assets_list = assets_result.unwrap_or_default();
    Html(templates::faucet_tab(q.account_id, assets_list))
}


// --- DETAIL & ACTION HANDLERS ---

#[derive(Deserialize)]
struct MarketDetailQuery {
    market_id: Uuid,
    account_id: Uuid,
}

async fn market_detail_handler(
    State(state): State<AppState>,
    Query(q): Query<MarketDetailQuery>,
) -> Html<String> {
    let input = MarketProcessorInput::GetMarket(q.market_id);
    let router_input = ActionRouterInput::Markets(input);
    
    let market_record = match call_action_router(router_input, (*state.config).clone()).await {
        Ok(ActionRouterOutput::Markets(cradle_back_end::market::processor_enums::MarketProcessorOutput::GetMarket(m))) => m,
        _ => return Html("<div>Error loading market details</div>".to_string())
    };

    use cradle_back_end::schema::orderbook::dsl as ob_dsl;
    use cradle_back_end::order_book::db_types::OrderBookRecord;
    use diesel::prelude::*;
    
    let pool = state.config.pool.clone();
    let acc_id = q.account_id;
    let m_id = q.market_id;
    
    let orders_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        ob_dsl::orderbook
            .filter(ob_dsl::market_id.eq(m_id))
            .order(ob_dsl::created_at.desc())
            .limit(20)
            .load::<OrderBookRecord>(&mut conn)
    }).await.unwrap();

    let orders = orders_result.unwrap_or_default();
    Html(templates::market_detail(market_record, q.account_id, orders))
}

#[derive(Deserialize)]
struct OnRampForm {
    account_id: Uuid,
    token: String,
    amount: String,
    email: String,
    result_page: Option<String>,
}

async fn on_ramp_handler(
    State(state): State<AppState>,
    Form(form): Form<OnRampForm>,
) -> Html<String> {
    eprintln!("[DEBUG] On-Ramp request: account_id={}, token={}, amount={}, email={}, result_page={:?}", 
        form.account_id, form.token, form.amount, form.email, form.result_page);
    // Logic from Ramper::onramp
    let ramper = match Ramper::from_env() {
        Ok(r) => r,
        Err(_) => return Html("<div class='text-red-400'>Failed to configure Ramper provider</div>".to_string())
    };

    let pool = state.config.pool.clone();
    // Getting raw conns for async ops usually requires deadpool or similar, but here we use the pool from AppConfig.
    // The existing ramper code takes `wallet: TaskWallet` which is `&mut dyn TaskWalletTrait`.
    // And `conn: DbConn` which is `&mut PooledConnection<...>`.
    // Accessing these in an async handler and passing to async function `ramper.onramp` requires care with lifetimes and ownership.
    
    // We'll try to follow `api/handlers/ramper.rs` pattern.
    
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Html("<div class='text-red-400'>Database connection failed</div>".to_string())
    };
    
    // `app_config.wallet` is `Arc<Box<dyn TaskWalletTrait>>` or similar? 
    // In `AppConfig`, `wallet` is `Box<dyn TaskWalletTrait + Send + Sync>`.
    // `state.config.wallet` is accessible.
    let mut wallet_ref = state.config.wallet.clone(); // This gives us the Box? `AppConfig` has `pub wallet: Box<...>`
    // Wait, `AppConfig` struct def: `pub wallet: Box<dyn TaskWalletTrait + Send + Sync>`.
    // Since it's a Box, cloning `AppConfig` generally not `Clone` unless implemented manually.
    // `AppConfig` IS NOT Clone? 
    // `AppState` has `config: Arc<AppConfig>`.
    // But `ramper.onramp` takes `&mut wallet`. We can't get mut ref from Arc.
    // However, `TaskWalletTrait` methods usually take `&self` or `&mut self`.
    // If they take `&mut self`, we are in trouble with Arc.
    // Let's check `api/handlers/ramper.rs`: `let mut wallet = app_config.wallet.clone();`
    // This implies `wallet` field or `AppConfig` is Clone? 
    // Or `app_config` in handler is `State<AppConfig>`. If State extracts `AppConfig`, it must be Clone.
    // `AppConfig` in `src/utils/app_config.rs` usually derives Clone or is Arc-wrapped internally?
    
    // If `AppConfig` is NOT Clone, then `State<AppConfig>` would fail if it wasn't wrapped in Arc in `router`.
    // In `router` I wrap it: `config: Arc::new(config)`.
    // `State` extractor gives me `State<AppState>`.
    // So `state.config` is `Arc<AppConfig>`.
    // I cannot clone `AppConfig` out of `Arc` if it's not Clone.
    // But `api` handlers seem to treat `AppConfig` as something they can clone wallet from?
    // Let's assume I can't easily clone the wallet box if I only have Arc.
    // BUT `api/handlers/ramper.rs` does: 
    // `State(app_config): State<AppConfig>` -> This implies `AppConfig` implements Clone!
    // If `AppConfig` implements Clone, then `state.config.as_ref().clone()` works.
    
    // Let's assume `AppConfig` is Clone.
    let app_config_clone = (*state.config).clone();
    let mut wallet = app_config_clone.wallet; 

    // Parse inputs
    let token_uuid = match Uuid::parse_str(&form.token) {
        Ok(u) => u,
        Err(_) => return Html("<div class='text-red-400'>Invalid Token UUID</div>".to_string())
    };
    let amount_decimal = match BigDecimal::from_str(&form.amount) {
        Ok(d) => d,
        Err(_) => return Html("<div class='text-red-400'>Invalid Amount</div>".to_string())
    };

    let req = OnRampRequest {
        token: token_uuid,
        amount: amount_decimal,
        wallet_id: form.account_id,
        result_page: form.result_page.unwrap_or_else(|| "http://localhost:3000/ui".to_string()),
        email: form.email,
    };

    eprintln!("[DEBUG] Calling ramper.onramp for wallet_id={}, token={}, amount={}", 
        req.wallet_id, req.token, req.amount);
    match ramper.onramp(&mut wallet, &mut conn, req).await {
        Ok(res) => {
            eprintln!("[DEBUG] On-ramp success: ref={}, url={}", res.reference, res.authorization_url);
            Html(format!(
            "<div class='bg-green-800 p-4 rounded text-green-200'>Success! Ref: {}<br><a href='{}' target='_blank' class='underline'>Pay Here</a></div>",
            res.reference, res.authorization_url
            ))
        },
        Err(e) => {
            eprintln!("[ERROR] On-ramp failed: {:?}", e);
            Html(format!("<div class='text-red-400'>On-Ramp Failed: {}</div>", e))
        }
    }
}

#[derive(Deserialize)]
struct FaucetForm {
    account_id: Uuid,
    asset_id: String,
}

async fn faucet_handler(
    State(state): State<AppState>,
    Form(form): Form<FaucetForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Faucet request: account_id={}, asset_id={}", form.account_id, form.asset_id);
    let pool = state.config.pool.clone();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Html("<div class='text-red-400'>Database connection failed</div>".to_string())
    };

    // Need mutable wallet from config. See notes in on_ramp_handler.
    let mut app_config_clone = (*state.config).clone();
    let mut action_wallet = app_config_clone.wallet; // Moves wallet out

    let asset_uuid = match Uuid::parse_str(&form.asset_id) {
        Ok(u) => u,
        Err(_) => return Html("<div class='text-red-400'>Invalid Asset UUID</div>".to_string())
    };

    // 1. Get Wallet Record
    let wallet_data = match get_wallet(&mut conn, form.account_id).await {
        Ok(w) => w,
        Err(_) => return Html("<div class='text-red-400'>Wallet not found</div>".to_string())
    };

    // 2. Get Asset Data
    let token_data = match get_asset(&mut conn, asset_uuid).await {
        Ok(t) => t,
        Err(_) => return Html("<div class='text-red-400'>Asset not found</div>".to_string())
    };

    // 3. Associate
    if let Err(e) = associate_token(
        &mut conn,
        &mut action_wallet,
        AssociateTokenToWalletInputArgs {
            wallet_id: wallet_data.id,
            token: token_data.id
        }
    ).await {
         return Html(format!("<div class='text-red-400'>Association failed: {}</div>", e));
    }

    // 4. KYC
    if let Err(e) = kyc_token(
        &mut conn,
        &mut action_wallet,
        GrantKYCInputArgs {
            wallet_id: wallet_data.id,
            token: token_data.id
        }
    ).await {
        return Html(format!("<div class='text-red-400'>KYC failed: {}</div>", e));
    }

    // 5. Mint
    let amount = 100_000_000_000_000u64; // Hardcoded large amount as per example
    if let Err(e) = mint_asset(
        &mut conn,
        &mut action_wallet,
        token_data.id,
        amount
    ).await {
        return Html(format!("<div class='text-red-400'>Minting failed: {}</div>", e));
    }

    // 6. Transfer/Airdrop (Contract Call)
    let airdrop_request = ContractCallInput::AssetManager(AssetManagerFunctionInput::Airdrop(AirdropArgs {
        amount: amount,
        asset_contract: token_data.asset_manager.clone(),
        target: wallet_data.address.clone(),
    }));

    eprintln!("[DEBUG] Calling airdrop contract function");
    match airdrop_request.process(&mut action_wallet).await {
        Ok(_) => {
            eprintln!("[DEBUG] Airdrop successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Airdrop Successful! Tokens sent.</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Airdrop failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Airdrop Contract Call Failed: {}</div>", e))
        }
    }
}

// Place Order Handler - Amount In/Out Style
// SEMANTICS (based on existing order_book implementation):
//   ask_asset = what you GIVE (asset_in) - this gets LOCKED
//   ask_amount = how much you GIVE (amount_in)
//   bid_asset = what you WANT (asset_out) - what you're bidding for
//   bid_amount = how much you WANT (amount_out)
//   price = ask_amount / bid_amount = amount_in / amount_out
#[derive(Deserialize, Debug)]
struct PlaceOrderForm {
    account_id: Uuid,
    market_id: Uuid,
    asset_in: Uuid,       // Asset you're giving (maps to ask_asset)
    asset_out: Uuid,      // Asset you're receiving (maps to bid_asset)
    amount_in: String,    // Amount you're giving (maps to ask_amount)
    amount_out: String,   // Amount you want to receive (maps to bid_amount)
    order_type: String,   // "limit" or "market"
    price: Option<String>, // Optional - for display/validation
}

async fn place_order_handler(
    State(state): State<AppState>,
    Form(form): Form<PlaceOrderForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Place order request: account_id={}, market_id={}",
        form.account_id, form.market_id);
    eprintln!("[DEBUG] asset_in (giving/ask)={}, amount_in={}", form.asset_in, form.amount_in);
    eprintln!("[DEBUG] asset_out (receiving/bid)={}, amount_out={}", form.asset_out, form.amount_out);
    eprintln!("[DEBUG] order_type={}, price={:?}", form.order_type, form.price);

    // Fetch asset records to get decimals
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    use diesel::prelude::*;

    let pool = state.config.pool.clone();
    let asset_in_id = form.asset_in;   // What you give = ask_asset
    let asset_out_id = form.asset_out; // What you want = bid_asset

    let assets_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let asset_in = ab_dsl::asset_book
            .find(asset_in_id)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        let asset_out = ab_dsl::asset_book
            .find(asset_out_id)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some((asset_in, asset_out))
    }).await.unwrap();

    let (asset_in_record, asset_out_record) = match assets_result {
        Some(assets) => assets,
        None => return Html("<div class='text-red-500'>Failed to fetch asset details</div>".to_string())
    };

    eprintln!("[DEBUG] Asset In (giving): {} (decimals: {}), Asset Out (receiving): {} (decimals: {})",
        asset_in_record.symbol, asset_in_record.decimals, asset_out_record.symbol, asset_out_record.decimals);

    // Parse amounts from form (user enters in human-readable format)
    let amount_in = BigDecimal::from_str(&form.amount_in).unwrap_or(BigDecimal::from(0));
    let amount_out = BigDecimal::from_str(&form.amount_out).unwrap_or(BigDecimal::from(0));

    if amount_in <= BigDecimal::from(0) || amount_out <= BigDecimal::from(0) {
        return Html("<div class='text-red-500'>Amount In and Amount Out must be greater than 0</div>".to_string());
    }

    // Scale amounts by their respective decimals
    // ask_amount = amount_in (scaled by asset_in decimals) - what you GIVE
    // bid_amount = amount_out (scaled by asset_out decimals) - what you WANT
    let ask_multiplier = BigDecimal::from(10i64.pow(asset_in_record.decimals as u32));
    let bid_multiplier = BigDecimal::from(10i64.pow(asset_out_record.decimals as u32));

    let ask_amt = amount_in.clone() * ask_multiplier;  // What you give
    let bid_amt = amount_out.clone() * bid_multiplier; // What you want

    // Calculate price as ask/bid ratio (amount_in / amount_out)
    let price = &amount_in / &amount_out;

    eprintln!("[DEBUG] Scaled amounts - ask_amt (giving): {}, bid_amt (wanting): {}, price: {}", ask_amt, bid_amt, price);

    use cradle_back_end::order_book::processor_enums::OrderBookProcessorInput;
    use cradle_back_end::order_book::db_types::{NewOrderBookRecord, OrderType as DbOrderType, FillMode};
    
    let o_type = match form.order_type.as_str() {
        "market" => DbOrderType::Market,
        _ => DbOrderType::Limit
    };

    // IMPORTANT: Mapping from amount_in/out to order book fields:
    // bid_asset = what you WANT to receive = asset_out
    // bid_amount = how much you WANT = amount_out (scaled)
    // ask_asset = what you GIVE = asset_in (this gets LOCKED)
    // ask_amount = how much you GIVE = amount_in (scaled)
    let new_order = NewOrderBookRecord {
        wallet: form.account_id,
        market_id: form.market_id,
        bid_asset: form.asset_out,  // What you WANT (bidding for)
        ask_asset: form.asset_in,   // What you GIVE (asking in exchange)
        bid_amount: bid_amt,        // How much you want
        ask_amount: ask_amt,        // How much you're giving
        price: price,
        mode: Some(FillMode::GoodTillCancel),
        expires_at: None,
        order_type: Some(o_type)
    };

    let input = OrderBookProcessorInput::PlaceOrder(new_order);
    let router_input = ActionRouterInput::OrderBook(input);

    eprintln!("[DEBUG] Submitting order to action router");
    match call_action_router(router_input, (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[DEBUG] Order submitted successfully");
            Html(format!(
                r#"<div class="bg-green-800 p-3 rounded text-green-200 text-sm">
                    Order Submitted!<br>
                    Giving: {} {}<br>
                    Receiving: {} {}
                </div>"#,
                form.amount_in, asset_in_record.symbol, form.amount_out, asset_out_record.symbol
            ))
        },
        Err(e) => {
            eprintln!("[ERROR] Order submission failed: {:?}", e);
            Html(format!(r#"<div class="bg-red-800 p-3 rounded text-red-200 text-sm">Error: {}</div>"#, e))
        }
    }
}
// Lending Form Structs
#[derive(Deserialize)]
struct SupplyForm {
    pool_id: Uuid,
    account_id: Uuid,
    amount: String,
}

#[derive(Deserialize)]
struct WithdrawForm {
    pool_id: Uuid,
    account_id: Uuid,
    amount: String,
}

#[derive(Deserialize)]
struct BorrowForm {
    pool_id: Uuid,
    account_id: Uuid,
    loan_amount: String,
    collateral_asset: String,
    collateral_price: String,
}

#[derive(Deserialize)]
struct RepayForm {
    account_id: Uuid,
    loan_id: Uuid,
    amount: String,
}

#[derive(Deserialize)]
struct QueryParams {
    pool_id: Option<Uuid>,
    account_id: Option<Uuid>,
    wallet_id: Option<Uuid>,
    listing_id: Option<Uuid>,
}

// Listing Form Structs
#[derive(Deserialize)]
struct CreateCompanyForm {
    account_id: Uuid,
    name: String,
    description: String,
    legal_documents: String,
}

#[derive(Deserialize)]
struct CreateListingForm {
    account_id: Uuid,
    company: String,
    name: String,
    description: String,
    listed_asset: String,
    purchase_asset: String,
    purchase_price: String,
    max_supply: String,
    documents: String,
}

#[derive(Deserialize)]
struct PurchaseListingForm {
    listing_id: Uuid,
    account_id: Uuid,
    amount: String,
}

#[derive(Deserialize)]
struct ReturnListingForm {
    listing_id: Uuid,
    account_id: Uuid,
    amount: String,
}

#[derive(Deserialize)]
struct WithdrawListingForm {
    listing_id: Uuid,
    account_id: Uuid,
    amount: String,
}

// Oracle Form Structs
#[derive(Deserialize)]
struct SetOraclePriceForm {
    pool_id: Uuid,
    asset_id: Uuid,
    price: String,
}

// Lending Handlers
async fn lending_tab_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[LENDING] Tab handler called - account_id: {:?}", params.account_id);
    let account_id = params.account_id.unwrap_or_default();
    use diesel::prelude::*;
    use cradle_back_end::schema::lendingpool::dsl::*;
    
    let pool = state.config.pool.clone();
    eprintln!("[LENDING] Fetching all pools from database");
    let pools = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        lendingpool.load::<LendingPoolRecord>(&mut conn).ok()
    }).await.unwrap().unwrap_or_default();
    
    eprintln!("[LENDING] Found {} pools", pools.len());
    Html(templates::lending_tab(account_id, pools))
}

async fn supply_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LENDING] Supply form requested - pool: {:?}, account: {:?}", params.pool_id, params.account_id);
    let pool_id = params.pool_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::supply_form(pool_id, account_id))
}

async fn borrow_form_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[LENDING] Borrow form requested - pool: {:?}, account: {:?}", params.pool_id, params.account_id);
    let pool_id = params.pool_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    
    use diesel::prelude::*;
    use cradle_back_end::schema::lendingpool::dsl as lp_dsl;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool = state.config.pool.clone();
    eprintln!("[LENDING] Fetching pool LTV and all assets");
    let (ltv, assets) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let pool_record = lp_dsl::lendingpool.find(pool_id).first::<LendingPoolRecord>(&mut conn).ok()?;
        let all_assets = ab_dsl::asset_book.load::<AssetBookRecord>(&mut conn).ok()?;
        Some((pool_record.loan_to_value.to_string(), all_assets))
    }).await.unwrap().unwrap_or_else(|| ("80".to_string(), vec![]));
    
    eprintln!("[LENDING] Pool LTV: {}, Assets available: {}", ltv, assets.len());
    Html(templates::borrow_form(pool_id, account_id, ltv, assets))
}

async fn withdraw_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LENDING] Withdraw form requested - pool: {:?}, account: {:?}", params.pool_id, params.account_id);
    let pool_id = params.pool_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::withdraw_form(pool_id, account_id))
}

async fn repay_form_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[LENDING] Repay form requested - account: {:?}", params.account_id);
    let account_id_param = params.account_id.unwrap_or_default();
    
    use diesel::prelude::*;
    use cradle_back_end::schema::loans::dsl::*;
    use cradle_back_end::lending_pool::db_types::LoanStatus;
    
    let pool_conn = state.config.pool.clone();
    eprintln!("[LENDING] Fetching active loans for wallet: {}", account_id_param);
    let active_loans = tokio::task::spawn_blocking(move || {
        let mut conn = pool_conn.get().ok()?;
        loans
            .filter(wallet_id.eq(account_id_param))
            .filter(status.eq(LoanStatus::Active))
            .load::<LoanRecord>(&mut conn).ok()
    }).await.unwrap().unwrap_or_default();
    
    eprintln!("[LENDING] Found {} active loans", active_loans.len());
    Html(templates::repay_form(account_id_param, active_loans))
}

async fn supply_liquidity_handler(
    State(state): State<AppState>,
    Form(form): Form<SupplyForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Supply liquidity: pool={}, account={}, amount={}", 
        form.pool_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{lendingpool::dsl as lp_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool_clone = state.config.pool.clone();
    let pool_id = form.pool_id;
    
    // Get reserve asset decimals
    eprintln!("[LENDING] Fetching reserve asset decimals for pool: {}", pool_id);
    let (reserve_asset_id, decimals) = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let pool = lp_dsl::lendingpool.find(pool_id).first::<LendingPoolRecord>(&mut conn).ok()?;
        let asset = ab_dsl::asset_book.find(pool.reserve_asset).first::<AssetBookRecord>(&mut conn).ok()?;
        Some((pool.reserve_asset, asset.decimals))
    }).await.unwrap() {
        Some(data) => data,
        None => return Html("<div class='text-red-400'>Failed to fetch pool/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = (amount * multiplier).to_u64().unwrap_or(0);
    
    eprintln!("[DEBUG] Scaled supply amount: {}", scaled_amount);
    
    let input = LendingPoolFunctionsInput::SupplyLiquidity(SupplyLiquidityInputArgs {
        wallet: form.account_id,
        pool: form.pool_id,
        amount: scaled_amount,
    });
    
    match call_action_router(ActionRouterInput::Pool(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[DEBUG] Supply successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Liquidity supplied successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Supply failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Supply failed: {}</div>", e))
        }
    }
}

async fn withdraw_liquidity_handler(
    State(state): State<AppState>,
    Form(form): Form<WithdrawForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Withdraw liquidity: pool={}, account={}, amount={}", 
        form.pool_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{lendingpool::dsl as lp_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool_clone = state.config.pool.clone();
    let pool_id = form.pool_id;
    
    // Get yield asset decimals
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let pool = lp_dsl::lendingpool.find(pool_id).first::<LendingPoolRecord>(&mut conn).ok()?;
        let asset = ab_dsl::asset_book.find(pool.yield_asset).first::<AssetBookRecord>(&mut conn).ok()?;
        Some(asset.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch pool/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = (amount * multiplier).to_u64().unwrap_or(0);
    
    eprintln!("[DEBUG] Scaled withdraw amount: {}", scaled_amount);
    
    let input = LendingPoolFunctionsInput::WithdrawLiquidity(WithdrawLiquidityInputArgs {
        wallet: form.account_id,
        pool: form.pool_id,
        amount: scaled_amount,
    });
    
    match call_action_router(ActionRouterInput::Pool(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[DEBUG] Withdraw successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Withdrawal successful!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Withdraw failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Withdrawal failed: {}</div>", e))
        }
    }
}

async fn borrow_handler(
    State(state): State<AppState>,
    Form(form): Form<BorrowForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Borrow: pool={}, account={}, loan_amount={}, collateral_asset={}, price={}", 
        form.pool_id, form.account_id, form.loan_amount, form.collateral_asset, form.collateral_price);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{lendingpool::dsl as lp_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool_clone = state.config.pool.clone();
    let pool_id = form.pool_id;
    let collateral_asset_uuid = match Uuid::from_str(&form.collateral_asset) {
        Ok(id) => id,
        Err(_) => return Html("<div class='text-red-400'>Invalid collateral asset ID</div>".to_string())
    };
    
    // Fetch pool, reserve asset, collateral asset
    let (ltv, reserve_decimals, collateral_decimals) = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let pool = lp_dsl::lendingpool.find(pool_id).first::<LendingPoolRecord>(&mut conn).ok()?;
        let reserve = ab_dsl::asset_book.find(pool.reserve_asset).first::<AssetBookRecord>(&mut conn).ok()?;
        let collateral = ab_dsl::asset_book.find(collateral_asset_uuid).first::<AssetBookRecord>(&mut conn).ok()?;
        Some((pool.loan_to_value, reserve.decimals, collateral.decimals))
    }).await.unwrap() {
        Some(data) => data,
        None => return Html("<div class='text-red-400'>Failed to fetch pool/asset data</div>".to_string())
    };
    
    eprintln!("[LENDING] Asset info - LTV: {}, Reserve decimals: {}, Collateral decimals: {}", 
        ltv, reserve_decimals, collateral_decimals);
    
    // Calculate amounts
    let loan_amount = BigDecimal::from_str(&form.loan_amount).unwrap_or_default();
    let price = BigDecimal::from_str(&form.collateral_price).unwrap_or_default();
    
    eprintln!("[LENDING] User input - Loan amount: {}, Collateral price: {}", loan_amount, price);
    
    // Calculate required collateral: ((10000/LTV) * loan_amount) / price
    // LTV is in basis points (7500 = 75%), so 10000 = 100%
    let collateral_multiplier = BigDecimal::from(10000) / ltv.clone();
    let required_collateral = (collateral_multiplier.clone() * loan_amount.clone()) / price.clone();
    eprintln!("[LENDING] Required collateral (unscaled): {} = ((10000/{}) * {}) / {}", 
        required_collateral, ltv, loan_amount, price);
    
    // Scale collateral amount with collateral asset decimals
    let collateral_multiplier = BigDecimal::from(10i64.pow(collateral_decimals as u32));
    let scaled_collateral = (required_collateral.clone() * collateral_multiplier.clone()).to_u64().unwrap_or(0);
    
    eprintln!("[LENDING] Scaled collateral amount: {} (multiplier: 10^{})", scaled_collateral, collateral_decimals);
    
    // TakeLoanInputArgs.amount is the collateral amount, not loan amount
    let input = LendingPoolFunctionsInput::BorrowAsset(TakeLoanInputArgs {
        wallet: form.account_id,
        pool: form.pool_id,
        amount: scaled_collateral,
        collateral: collateral_asset_uuid,
    });
    
    match call_action_router(ActionRouterInput::Pool(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[DEBUG] Borrow successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Loan taken successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Borrow failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Borrow failed: {}</div>", e))
        }
    }
}

async fn repay_handler(
    State(state): State<AppState>,
    Form(form): Form<RepayForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Repay: loan={}, account={}, amount={}", 
        form.loan_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{loans::dsl as loan_dsl, lendingpool::dsl as lp_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool_clone = state.config.pool.clone();
    let loan_id = form.loan_id;
    
    // Get loan and reserve asset decimals
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let loan = loan_dsl::loans.find(loan_id).first::<LoanRecord>(&mut conn).ok()?;
        let pool = lp_dsl::lendingpool.find(loan.pool).first::<LendingPoolRecord>(&mut conn).ok()?;
        let asset = ab_dsl::asset_book.find(pool.reserve_asset).first::<AssetBookRecord>(&mut conn).ok()?;
        Some(asset.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch loan/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = (amount * multiplier).to_u64().unwrap_or(0);
    
    eprintln!("[DEBUG] Scaled repay amount: {}", scaled_amount);
    
    let input = LendingPoolFunctionsInput::RepayBorrow(RepayLoanInputArgs {
        wallet: form.account_id,
        loan: form.loan_id,
        amount: scaled_amount,
    });
    
    match call_action_router(ActionRouterInput::Pool(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[DEBUG] Repay successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Loan repayment successful!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Repay failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Repayment failed: {}</div>", e))
        }
    }
}

async fn pool_stats_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    let pool_id = match params.pool_id {
        Some(id) => id,
        None => return Html("<p class='text-gray-400'>No pool selected</p>".to_string())
    };
    
    let pool_clone = state.config.pool.clone();
    let mut wallet = state.config.wallet.clone();
    
    let mut conn = match pool_clone.get() {
        Ok(c) => c,
        Err(_) => return Html("<p class='text-red-400'>Database error</p>".to_string())
    };
    
    eprintln!("[LENDING] Fetching pool stats for pool: {}", pool_id);
    match get_pool_stats(&mut wallet, &mut conn, pool_id).await {
        Ok(stats) => {
            eprintln!("[LENDING] Pool stats retrieved - Supply: {}, Borrow: {}, Util: {}%", 
                stats.total_supplied, stats.total_borrowed, stats.utilization);
            Html(format!(r##"
                <div class="grid grid-cols-2 gap-4">
                    <div><p class="text-gray-400">Total Supply</p><p class="text-2xl font-bold text-white">{}</p></div>
                    <div><p class="text-gray-400">Total Borrow</p><p class="text-2xl font-bold text-white">{}</p></div>
                    <div><p class="text-gray-400">Supply APY</p><p class="text-2xl font-bold text-green-400">{}%</p></div>
                    <div><p class="text-gray-400">Borrow APY</p><p class="text-2xl font-bold text-red-400">{}%</p></div>
                    <div><p class="text-gray-400">Utilization</p><p class="text-2xl font-bold text-blue-400">{}%</p></div>
                    <div><p class="text-gray-400">Available</p><p class="text-2xl font-bold text-white">{}</p></div>
                </div>
            "##, stats.total_supplied, stats.total_borrowed, stats.supply_rate, stats.borrow_rate, 
                stats.utilization, stats.liquidity))
        },
        Err(e) => {
            eprintln!("[ERROR] Failed to get pool stats: {:?}", e);
            Html(format!("<p class='text-red-400'>Failed to load stats: {}</p>", e))
        }
    }
}

async fn user_positions_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    let pool_id_param = match params.pool_id {
        Some(id_val) => id_val,
        None => return Html("<p class='text-gray-400'>No pool selected</p>".to_string())
    };
    let wallet_id_param = match params.wallet_id {
        Some(id_val) => id_val,
        None => return Html("<p class='text-gray-400'>No wallet specified</p>".to_string())
    };
    
    let pool_clone = state.config.pool.clone();
    let mut wallet = state.config.wallet.clone();
    let mut conn = match pool_clone.get() {
        Ok(c) => c,
        Err(_) => return Html("<p class='text-red-400'>Database error</p>".to_string())
    };
    
    eprintln!("[LENDING] Fetching user positions - pool: {}, wallet: {}", pool_id_param, wallet_id_param);
    
    // Get deposit position
    eprintln!("[LENDING] Fetching deposit position");
    let deposit_html = match get_pool_deposit_position(&mut wallet, &mut conn, pool_id_param, wallet_id_param).await {
        Ok(pos) => format!("<p class='text-green-400'>Deposited: {} (Underlying: {})</p>", 
            pos.yield_token_balance, pos.underlying_value),
        Err(_) => "<p class='text-gray-500'>No deposits</p>".to_string()
    };
    
    // Get active loans
    use diesel::prelude::*;
    use cradle_back_end::schema::loans::dsl::*;
    use cradle_back_end::lending_pool::db_types::LoanStatus;
    
    let loan_records = loans
        .filter(wallet_id.eq(wallet_id_param))
        .filter(pool.eq(pool_id_param))
        .filter(status.eq(LoanStatus::Active))
        .load::<LoanRecord>(&mut conn)
        .unwrap_or_default();
    
    eprintln!("[LENDING] Found {} active loans for this pool", loan_records.len());
    
    let loans_html = if loan_records.is_empty() {
        "<p class='text-gray-500'>No active loans</p>".to_string()
    } else {
        loan_records.iter().map(|l| 
            format!("<p class='text-yellow-400'>Loan {}: Principal {}</p>", l.id, l.principal_amount)
        ).collect::<Vec<_>>().join("")
    };
    
    Html(format!(r##"
        <div class="space-y-4">
            <div><h4 class="font-bold text-white mb-2">Deposits</h4>{}</div>
            <div><h4 class="font-bold text-white mb-2">Loans</h4>{}</div>
        </div>
    "##, deposit_html, loans_html))
}

// Listing Handlers
async fn listings_tab_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[LISTINGS] Tab handler called - account_id: {:?}", params.account_id);
    let account_id = params.account_id.unwrap_or_default();
    
    use diesel::prelude::*;
    use cradle_back_end::schema::cradlenativelistings::dsl as listings_dsl;
    use cradle_back_end::schema::cradlelistedcompanies::dsl as companies_dsl;
    
    let pool = state.config.pool.clone();
    eprintln!("[LISTINGS] Fetching all listings and companies from database");
    
    let (listings, companies) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let all_listings = listings_dsl::cradlenativelistings
            .load::<CradleNativeListingRow>(&mut conn)
            .ok()?;
        let all_companies = companies_dsl::cradlelistedcompanies
            .load::<CompanyRow>(&mut conn)
            .ok()?;
        Some((all_listings, all_companies))
    }).await.unwrap().unwrap_or((vec![], vec![]));
    
    eprintln!("[LISTINGS] Found {} listings and {} companies", listings.len(), companies.len());
    Html(templates::listings_tab(account_id, listings, companies))
}

async fn create_company_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LISTINGS] Create company form requested - account: {:?}", params.account_id);
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::create_company_form(account_id))
}

async fn create_listing_form_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[LISTINGS] Create listing form requested - account: {:?}", params.account_id);
    let account_id = params.account_id.unwrap_or_default();
    
    use diesel::prelude::*;
    use cradle_back_end::schema::cradlelistedcompanies::dsl as companies_dsl;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool = state.config.pool.clone();
    eprintln!("[LISTINGS] Fetching companies and assets");
    
    let (companies, assets) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let all_companies = companies_dsl::cradlelistedcompanies
            .load::<CompanyRow>(&mut conn)
            .ok()?;
        let all_assets = ab_dsl::asset_book
            .load::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some((all_companies, all_assets))
    }).await.unwrap().unwrap_or((vec![], vec![]));
    
    eprintln!("[LISTINGS] Found {} companies and {} assets", companies.len(), assets.len());
    Html(templates::create_listing_form(account_id, companies, assets))
}

async fn purchase_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LISTINGS] Purchase form requested - account: {:?}, listing: {:?}", params.account_id, params.listing_id);
    let listing_id = params.listing_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::purchase_listing_form(listing_id, account_id))
}

async fn return_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LISTINGS] Return form requested - account: {:?}, listing: {:?}", params.account_id, params.listing_id);
    let listing_id = params.listing_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::return_listing_form(listing_id, account_id))
}

async fn withdraw_listing_form_handler(Query(params): Query<QueryParams>) -> Html<String> {
    eprintln!("[LISTINGS] Withdraw form requested - account: {:?}, listing: {:?}", params.account_id, params.listing_id);
    let listing_id = params.listing_id.unwrap_or_default();
    let account_id = params.account_id.unwrap_or_default();
    Html(templates::withdraw_listing_form(listing_id, account_id))
}

async fn create_company_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateCompanyForm>,
) -> Html<String> {
    eprintln!("[LISTINGS] Creating company: name={}, account={}", form.name, form.account_id);
    
    let input = CradleNativeListingFunctionsInput::CreateCompany(CreateCompanyInputArgs {
        name: form.name.clone(),
        description: form.description,
        legal_documents: form.legal_documents,
    });
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[LISTINGS] Company created successfully: {}", form.name);
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Company created successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Company creation failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Company creation failed: {}</div>", e))
        }
    }
}

async fn create_listing_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateListingForm>,
) -> Html<String> {
    eprintln!("[LISTINGS] Creating listing: name={}, company={}", form.name, form.company);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    // Parse UUIDs
    let company_uuid = match Uuid::from_str(&form.company) {
        Ok(id) => id,
        Err(_) => return Html("<div class='text-red-400'>Invalid company ID</div>".to_string())
    };
    
    let listed_asset_uuid = match Uuid::from_str(&form.listed_asset) {
        Ok(id) => id,
        Err(_) => return Html("<div class='text-red-400'>Invalid listed asset ID</div>".to_string())
    };
    
    let purchase_asset_uuid = match Uuid::from_str(&form.purchase_asset) {
        Ok(id) => id,
        Err(_) => return Html("<div class='text-red-400'>Invalid purchase asset ID</div>".to_string())
    };
    
    // Get asset decimals for scaling
    let pool_clone = state.config.pool.clone();
    let (listed_decimals, purchase_decimals) = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let listed = ab_dsl::asset_book.find(listed_asset_uuid).first::<AssetBookRecord>(&mut conn).ok()?;
        let purchase = ab_dsl::asset_book.find(purchase_asset_uuid).first::<AssetBookRecord>(&mut conn).ok()?;
        Some((listed.decimals, purchase.decimals))
    }).await.unwrap() {
        Some(data) => data,
        None => return Html("<div class='text-red-400'>Failed to fetch asset data</div>".to_string())
    };
    
    // Parse and scale amounts
    let purchase_price = BigDecimal::from_str(&form.purchase_price).unwrap_or_default();
    let max_supply = BigDecimal::from_str(&form.max_supply).unwrap_or_default();
    
    let price_multiplier = BigDecimal::from(10i64.pow(purchase_decimals as u32));
    let supply_multiplier = BigDecimal::from(10i64.pow(listed_decimals as u32));
    
    let scaled_price = purchase_price * price_multiplier;
    let scaled_supply = max_supply * supply_multiplier;
    
    eprintln!("[LISTINGS] Scaled price: {}, scaled supply: {}", scaled_price, scaled_supply);
    
    let input = CradleNativeListingFunctionsInput::CreateListing(CreateListingInputArgs {
        name: form.name.clone(),
        description: form.description,
        documents: form.documents,
        company: company_uuid,
        asset: AssetDetails::Existing(listed_asset_uuid),
        purchase_asset: purchase_asset_uuid,
        purchase_price: scaled_price,
        max_supply: scaled_supply,
    });
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[LISTINGS] Listing created successfully: {}", form.name);
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Listing created successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Listing creation failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Listing creation failed: {}</div>", e))
        }
    }
}

async fn purchase_listing_handler(
    State(state): State<AppState>,
    Form(form): Form<PurchaseListingForm>,
) -> Html<String> {
    eprintln!("[LISTINGS] Purchase request: listing={}, account={}, amount={}", 
        form.listing_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{cradlenativelistings::dsl as listings_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    // Get listing and asset decimals
    let pool_clone = state.config.pool.clone();
    let listing_id = form.listing_id;
    
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let listing = listings_dsl::cradlenativelistings
            .find(listing_id)
            .first::<CradleNativeListingRow>(&mut conn)
            .ok()?;
        let asset = ab_dsl::asset_book
            .find(listing.listed_asset)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some(asset.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch listing/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = amount * multiplier;
    
    eprintln!("[LISTINGS] Scaled purchase amount: {} (10^{})", scaled_amount, decimals);
    
    let input = CradleNativeListingFunctionsInput::Purchase(PurchaseListingAssetInputArgs {
        wallet: form.account_id,
        amount: scaled_amount,
        listing: form.listing_id,
    });
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[LISTINGS] Purchase successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Purchase successful!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Purchase failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Purchase failed: {}</div>", e))
        }
    }
}

async fn return_listing_handler(
    State(state): State<AppState>,
    Form(form): Form<ReturnListingForm>,
) -> Html<String> {
    eprintln!("[LISTINGS] Return request: listing={}, account={}, amount={}", 
        form.listing_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{cradlenativelistings::dsl as listings_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    // Get listing and asset decimals
    let pool_clone = state.config.pool.clone();
    let listing_id = form.listing_id;
    
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let listing = listings_dsl::cradlenativelistings
            .find(listing_id)
            .first::<CradleNativeListingRow>(&mut conn)
            .ok()?;
        let asset = ab_dsl::asset_book
            .find(listing.listed_asset)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some(asset.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch listing/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = amount * multiplier;
    
    eprintln!("[LISTINGS] Scaled return amount: {} (10^{})", scaled_amount, decimals);
    
    let input = CradleNativeListingFunctionsInput::ReturnAsset(ReturnAssetListingInputArgs {
        wallet: form.account_id,
        amount: scaled_amount,
        listing: form.listing_id,
    });
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[LISTINGS] Return successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Return successful!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Return failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Return failed: {}</div>", e))
        }
    }
}

async fn withdraw_listing_handler(
    State(state): State<AppState>,
    Form(form): Form<WithdrawListingForm>,
) -> Html<String> {
    eprintln!("[LISTINGS] Withdraw request: listing={}, account={}, amount={}", 
        form.listing_id, form.account_id, form.amount);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{cradlenativelistings::dsl as listings_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    // Get listing and purchase asset decimals for withdrawal
    let pool_clone = state.config.pool.clone();
    let listing_id = form.listing_id;
    
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let listing = listings_dsl::cradlenativelistings
            .find(listing_id)
            .first::<CradleNativeListingRow>(&mut conn)
            .ok()?;
        let asset = ab_dsl::asset_book
            .find(listing.purchase_with_asset)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some(asset.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch listing/asset data</div>".to_string())
    };
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_amount = amount * multiplier;
    
    eprintln!("[LISTINGS] Scaled withdraw amount: {} (10^{})", scaled_amount, decimals);
    
    let input = CradleNativeListingFunctionsInput::WithdrawToBeneficiary(WithdrawToBeneficiaryInputArgsBody {
        amount: scaled_amount,
        listing: form.listing_id,
    });
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[LISTINGS] Withdrawal successful");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Withdrawal to beneficiary successful!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Withdrawal failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Withdrawal failed: {}</div>", e))
        }
    }
}

async fn listing_stats_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    let listing_id = match params.listing_id.or(params.pool_id) {
        Some(id) => id,
        None => return Html("<p class='text-gray-400'>No listing selected</p>".to_string())
    };
    
    eprintln!("[LISTINGS] Fetching stats for listing: {}", listing_id);
    
    // Call GetStats via action router
    let input = CradleNativeListingFunctionsInput::GetStats(listing_id);
    
    match call_action_router(ActionRouterInput::Listing(input), (*state.config).clone()).await {
        Ok(ActionRouterOutput::Listing(_output)) => {
            // For now, return a simple success message
            // In a real implementation, you'd parse the output and display the stats
            eprintln!("[LISTINGS] Stats retrieved successfully");
            Html(r##"
                <div class="grid grid-cols-2 gap-4">
                    <div><p class="text-gray-400">Total Purchased</p><p class="text-2xl font-bold text-white">Loading...</p></div>
                    <div><p class="text-gray-400">Total Supply</p><p class="text-2xl font-bold text-white">Loading...</p></div>
                    <div><p class="text-gray-400">Status</p><p class="text-2xl font-bold text-green-400">Active</p></div>
                </div>
            "##.to_string())
        },
        Ok(_) => {
            eprintln!("[LISTINGS] Unexpected output type from action router");
            Html("<p class='text-red-400'>Unexpected response format</p>".to_string())
        },
        Err(e) => {
            eprintln!("[LISTINGS] Failed to get stats: {:?}", e);
            Html(format!("<p class='text-red-400'>Failed to load stats: {}</p>", e))
        }
    }
}

// Oracle Handlers
async fn oracle_tab_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Html<String> {
    eprintln!("[ORACLE] Tab handler called - account_id: {:?}", params.account_id);
    let account_id = params.account_id.unwrap_or_default();
    
    use diesel::prelude::*;
    use cradle_back_end::schema::lendingpool::dsl as lp_dsl;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    let pool = state.config.pool.clone();
    eprintln!("[ORACLE] Fetching pools and assets from database");
    
    let (pools, assets) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let all_pools = lp_dsl::lendingpool
            .load::<LendingPoolRecord>(&mut conn)
            .ok()?;
        let all_assets = ab_dsl::asset_book
            .load::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some((all_pools, all_assets))
    }).await.unwrap().unwrap_or((vec![], vec![]));
    
    eprintln!("[ORACLE] Found {} pools and {} assets", pools.len(), assets.len());
    Html(templates::oracle_tab(account_id, pools, assets))
}

async fn set_oracle_price_handler(
    State(state): State<AppState>,
    Form(form): Form<SetOraclePriceForm>,
) -> Html<String> {
    eprintln!("[ORACLE] Set price request: pool={}, asset={}, price={}", 
        form.pool_id, form.asset_id, form.price);
    
    use diesel::prelude::*;
    use cradle_back_end::schema::{lendingpool::dsl as lp_dsl, asset_book::dsl as ab_dsl};
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    
    // Get pool and reserve asset to determine decimals
    let pool_clone = state.config.pool.clone();
    let pool_id = form.pool_id;
    
    let decimals = match tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        let pool = lp_dsl::lendingpool
            .find(pool_id)
            .first::<LendingPoolRecord>(&mut conn)
            .ok()?;
        let reserve = ab_dsl::asset_book
            .find(pool.reserve_asset)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some(reserve.decimals)
    }).await.unwrap() {
        Some(d) => d,
        None => return Html("<div class='text-red-400'>Failed to fetch pool/reserve asset data</div>".to_string())
    };
    
    // Parse and scale price
    let price = BigDecimal::from_str(&form.price).unwrap_or_default();
    let multiplier = BigDecimal::from(10i64.pow(decimals as u32));
    let scaled_price = price * multiplier;
    
    eprintln!("[ORACLE] Scaled price: {} (multiplier: 10^{})", scaled_price, decimals);
    
    // Get DB connection and wallet
    let mut app_config_clone = (*state.config).clone();
    let mut wallet = app_config_clone.wallet;
    let pool_db = state.config.pool.clone();
    let mut conn = match pool_db.get() {
        Ok(c) => c,
        Err(_) => return Html("<div class='text-red-400'>Database connection failed</div>".to_string())
    };
    
    // Call oracle::publish_price
    eprintln!("[ORACLE] Publishing price to oracle contract...");
    match publish_price(&mut conn, &mut wallet, form.pool_id, form.asset_id, scaled_price).await {
        Ok(_) => {
            eprintln!("[ORACLE] Price published successfully");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Oracle price updated successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ORACLE] Price publication failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Failed to update oracle price: {}</div>", e))
        }
    }
}

// ============================================================================
// GLOBAL ADMIN TOOLS (No account selection required)
// ============================================================================

async fn admin_tools_handler() -> Html<String> {
    Html(templates::admin_tools_page())
}

async fn admin_assets_tab_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::asset_book::dsl::*;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;

    let pool = state.config.pool.clone();
    eprintln!("[ADMIN] Fetching all assets");

    let assets_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        asset_book.load::<AssetBookRecord>(&mut conn)
    }).await.unwrap();

    let assets_list = assets_result.unwrap_or_default();
    Html(templates::admin_assets_tab(assets_list))
}

async fn admin_markets_tab_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::markets::dsl::*;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;

    let pool = state.config.pool.clone();
    eprintln!("[ADMIN] Fetching all markets and assets");

    let (markets_list, assets_list) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        let m = markets.load::<MarketRecord>(&mut conn).unwrap_or_default();
        let a = ab_dsl::asset_book.load::<AssetBookRecord>(&mut conn).unwrap_or_default();
        (m, a)
    }).await.unwrap();

    Html(templates::admin_markets_tab(markets_list, assets_list))
}

async fn admin_aggregator_tab_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::markets::dsl::*;
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;

    let pool = state.config.pool.clone();
    eprintln!("[ADMIN] Fetching markets for aggregator");

    let (markets_list, assets_list) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        let m = markets.load::<MarketRecord>(&mut conn).unwrap_or_default();
        let a = ab_dsl::asset_book.load::<AssetBookRecord>(&mut conn).unwrap_or_default();
        (m, a)
    }).await.unwrap();

    Html(templates::admin_aggregator_tab(markets_list, assets_list))
}

// Asset Form Handlers
async fn create_new_asset_form_handler() -> Html<String> {
    Html(templates::create_new_asset_form())
}

async fn create_existing_asset_form_handler() -> Html<String> {
    Html(templates::create_existing_asset_form())
}

// Asset Creation Form Structs
#[derive(Deserialize)]
struct CreateNewAssetForm {
    name: String,
    symbol: String,
    decimals: i32,
    asset_type: String,
    icon: Option<String>,
}

#[derive(Deserialize)]
struct CreateExistingAssetForm {
    token: String,
    asset_manager: Option<String>,
    name: String,
    symbol: String,
    decimals: i32,
    asset_type: String,
    icon: Option<String>,
}

async fn create_new_asset_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateNewAssetForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Creating new asset: name={}, symbol={}, decimals={}",
        form.name, form.symbol, form.decimals);

    let asset_type = match form.asset_type.as_str() {
        "bridged" => AssetType::Bridged,
        "native" => AssetType::Native,
        "yield_bearing" => AssetType::YieldBearing,
        "chain_native" => AssetType::ChainNative,
        "stablecoin" => AssetType::StableCoin,
        "volatile" => AssetType::Volatile,
        _ => AssetType::Native,
    };

    let input = AssetBookProcessorInput::CreateNewAsset(CreateNewAssetInputArgs {
        asset_type,
        name: form.name.clone(),
        symbol: form.symbol.clone(),
        decimals: form.decimals,
        icon: form.icon.unwrap_or_default(),
    });

    match call_action_router(ActionRouterInput::AssetBook(input), (*state.config).clone()).await {
        Ok(ActionRouterOutput::AssetBook(cradle_back_end::asset_book::processor_enums::AssetBookProcessorOutput::CreateNewAsset(id))) => {
            eprintln!("[ADMIN] Asset created successfully: {}", id);
            Html(format!("<div class='bg-green-800 p-4 rounded text-green-200'>Asset created successfully!<br>ID: {}</div>", id))
        },
        Ok(_) => Html("<div class='text-red-400'>Unexpected response format</div>".to_string()),
        Err(e) => {
            eprintln!("[ADMIN] Asset creation failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Asset creation failed: {}</div>", e))
        }
    }
}

async fn create_existing_asset_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateExistingAssetForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Registering existing asset: token={}, name={}, symbol={}",
        form.token, form.name, form.symbol);

    let asset_type = match form.asset_type.as_str() {
        "bridged" => AssetType::Bridged,
        "native" => AssetType::Native,
        "yield_bearing" => AssetType::YieldBearing,
        "chain_native" => AssetType::ChainNative,
        "stablecoin" => AssetType::StableCoin,
        "volatile" => AssetType::Volatile,
        _ => AssetType::Native,
    };

    let input = AssetBookProcessorInput::CreateExistingAsset(CreateExistingAssetInputArgs {
        asset_manager: form.asset_manager,
        token: form.token.clone(),
        asset_type,
        name: form.name.clone(),
        symbol: form.symbol.clone(),
        decimals: form.decimals,
        icon: form.icon.unwrap_or_default(),
    });

    match call_action_router(ActionRouterInput::AssetBook(input), (*state.config).clone()).await {
        Ok(ActionRouterOutput::AssetBook(cradle_back_end::asset_book::processor_enums::AssetBookProcessorOutput::CreateExistingAsset(id))) => {
            eprintln!("[ADMIN] Existing asset registered successfully: {}", id);
            Html(format!("<div class='bg-green-800 p-4 rounded text-green-200'>Asset registered successfully!<br>ID: {}</div>", id))
        },
        Ok(_) => Html("<div class='text-red-400'>Unexpected response format</div>".to_string()),
        Err(e) => {
            eprintln!("[ADMIN] Existing asset registration failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Asset registration failed: {}</div>", e))
        }
    }
}

// Market Form Handlers
async fn create_market_form_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::asset_book::dsl::*;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;

    let pool = state.config.pool.clone();
    let assets_list = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        asset_book.load::<AssetBookRecord>(&mut conn).unwrap_or_default()
    }).await.unwrap();

    Html(templates::create_market_form(assets_list))
}

#[derive(Deserialize)]
struct CreateMarketForm {
    name: String,
    description: Option<String>,
    asset_one: Uuid,
    asset_two: Uuid,
    market_type: String,
    market_regulation: String,
}

async fn create_market_handler(
    State(state): State<AppState>,
    Form(form): Form<CreateMarketForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Creating market: name={}, asset_one={}, asset_two={}",
        form.name, form.asset_one, form.asset_two);

    let market_type = match form.market_type.as_str() {
        "spot" => MarketType::Spot,
        "derivative" => MarketType::Derivative,
        "futures" => MarketType::Futures,
        _ => MarketType::Spot,
    };

    let market_regulation = match form.market_regulation.as_str() {
        "regulated" => MarketRegulation::Regulated,
        "unregulated" => MarketRegulation::Unregulated,
        _ => MarketRegulation::Unregulated,
    };

    let create_input = CreateMarket {
        name: form.name.clone(),
        description: form.description,
        icon: None,
        asset_one: form.asset_one,
        asset_two: form.asset_two,
        market_type: Some(market_type),
        market_status: Some(MarketStatus::Active),
        market_regulation: Some(market_regulation),
    };

    let input = MarketProcessorInput::CreateMarket(create_input);

    match call_action_router(ActionRouterInput::Markets(input), (*state.config).clone()).await {
        Ok(ActionRouterOutput::Markets(cradle_back_end::market::processor_enums::MarketProcessorOutput::CreateMarket(id))) => {
            eprintln!("[ADMIN] Market created successfully: {}", id);
            Html(format!("<div class='bg-green-800 p-4 rounded text-green-200'>Market created successfully!<br>ID: {}</div>", id))
        },
        Ok(_) => Html("<div class='text-red-400'>Unexpected response format</div>".to_string()),
        Err(e) => {
            eprintln!("[ADMIN] Market creation failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Market creation failed: {}</div>", e))
        }
    }
}

#[derive(Deserialize)]
struct UpdateMarketStatusForm {
    market_id: Uuid,
    status: String,
}

async fn update_market_status_handler(
    State(state): State<AppState>,
    Form(form): Form<UpdateMarketStatusForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Updating market status: market_id={}, status={}",
        form.market_id, form.status);

    let status = match form.status.as_str() {
        "active" => MarketStatus::Active,
        "inactive" => MarketStatus::InActive,
        "suspended" => MarketStatus::Suspended,
        _ => MarketStatus::Active,
    };

    use cradle_back_end::market::processor_enums::UpdateMarketStatusInputArgs;
    let input = MarketProcessorInput::UpdateMarketStatus(UpdateMarketStatusInputArgs {
        market_id: form.market_id,
        status,
    });

    match call_action_router(ActionRouterInput::Markets(input), (*state.config).clone()).await {
        Ok(_) => {
            eprintln!("[ADMIN] Market status updated successfully");
            Html("<div class='bg-green-800 p-4 rounded text-green-200'>Market status updated successfully!</div>".to_string())
        },
        Err(e) => {
            eprintln!("[ADMIN] Market status update failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Status update failed: {}</div>", e))
        }
    }
}

// Aggregator Handlers
#[derive(Deserialize)]
struct RunAggregatorForm {
    market_id: Uuid,
    asset_id: Uuid,
    interval: String,
    duration: String,
}

async fn aggregator_markets_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::markets::dsl::*;

    let pool = state.config.pool.clone();
    let markets_list = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        markets.load::<MarketRecord>(&mut conn).unwrap_or_default()
    }).await.unwrap();

    // Build a simple JSON-like response with market assets
    let mut options = String::new();
    for m in markets_list {
        options.push_str(&format!(
            r##"<option value="{}" data-asset-one="{}" data-asset-two="{}">{}</option>"##,
            m.id, m.asset_one, m.asset_two, m.name
        ));
    }
    Html(options)
}

async fn run_aggregator_handler(
    State(state): State<AppState>,
    Form(form): Form<RunAggregatorForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Running aggregator: market={}, asset={}, interval={}, duration={}",
        form.market_id, form.asset_id, form.interval, form.duration);

    use diesel::prelude::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, JoinOnDsl, BoolExpressionMethods};
    use cradle_back_end::schema::orderbooktrades;
    use cradle_back_end::schema::orderbook;

    let pool_clone = state.config.pool.clone();

    // Parse interval
    let interval_enum = match form.interval.as_str() {
        "15secs" => TimeSeriesInterval::FifteenSecs,
        "30secs" => TimeSeriesInterval::ThirtySecs,
        "45secs" => TimeSeriesInterval::FortyFiveSecs,
        "1min" => TimeSeriesInterval::OneMinute,
        "5min" => TimeSeriesInterval::FiveMinutes,
        "15min" => TimeSeriesInterval::FifteenMinutes,
        "30min" => TimeSeriesInterval::ThirtyMinutes,
        "1hr" => TimeSeriesInterval::OneHour,
        "4hr" => TimeSeriesInterval::FourHours,
        "1day" => TimeSeriesInterval::OneDay,
        "1week" => TimeSeriesInterval::OneWeek,
        _ => TimeSeriesInterval::FifteenMinutes,
    };

    let interval_duration = match form.interval.as_str() {
        "15secs" => Duration::seconds(15),
        "30secs" => Duration::seconds(30),
        "45secs" => Duration::seconds(45),
        "1min" => Duration::minutes(1),
        "5min" => Duration::minutes(5),
        "15min" => Duration::minutes(15),
        "30min" => Duration::minutes(30),
        "1hr" => Duration::hours(1),
        "4hr" => Duration::hours(4),
        "1day" => Duration::days(1),
        "1week" => Duration::weeks(1),
        _ => Duration::minutes(15),
    };

    // Parse duration
    let now = Local::now().naive_local();
    let start_time = match form.duration.as_str() {
        "24h" => now - Duration::days(1),
        "7d" => now - Duration::days(7),
        "30d" => now - Duration::days(30),
        "90d" => now - Duration::days(90),
        "all" => now - Duration::days(36500), // ~100 years
        _ => now - Duration::days(7),
    };
    let end_time = now;

    let market_id = form.market_id;
    let asset_id = form.asset_id;

    // Query trades
    let trades_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        orderbooktrades::table
            .inner_join(orderbook::table.on(
                orderbooktrades::maker_order_id.eq(orderbook::id)
            ))
            .filter(
                orderbook::market_id.eq(market_id)
                    .and(orderbooktrades::created_at.ge(start_time))
                    .and(orderbooktrades::created_at.le(end_time))
            )
            .select(orderbooktrades::all_columns)
            .order_by(orderbooktrades::created_at.asc())
            .load::<OrderBookTradeRecord>(&mut conn)
            .ok()
    }).await.unwrap();

    let trades = match trades_result {
        Some(t) => t,
        None => return Html("<div class='text-red-400'>Failed to query trades</div>".to_string())
    };

    if trades.is_empty() {
        return Html("<div class='text-yellow-400'>No trades found for the specified time range</div>".to_string());
    }

    eprintln!("[ADMIN] Found {} trades to aggregate", trades.len());

    // Calculate OHLC bars
    let bars = calculate_ohlc_bars_for_admin(&trades, start_time, end_time, interval_duration);

    let mut bar_count = 0;
    let mut errors = 0;

    for (bar_start, bar_end, bar) in bars {
        let create_input = CreateMarketTimeSeriesRecord {
            market_id,
            asset: asset_id,
            open: bar.open.clone(),
            high: bar.high.clone(),
            low: bar.low.clone(),
            close: bar.close.clone(),
            volume: bar.volume.clone(),
            start_time: bar_start,
            end_time: bar_end,
            interval: Some(interval_enum.clone()),
            data_provider_type: Some(DataProviderType::OrderBook),
            data_provider: None,
        };

        let input = MarketTimeSeriesProcessorInput::AddRecord(create_input);
        let router_input = ActionRouterInput::MarketTimeSeries(input);

        match call_action_router(router_input, (*state.config).clone()).await {
            Ok(_) => bar_count += 1,
            Err(e) => {
                eprintln!("[ADMIN] Failed to create OHLC record: {:?}", e);
                errors += 1;
            }
        }
    }

    eprintln!("[ADMIN] Aggregation complete: {} bars created, {} errors", bar_count, errors);

    if errors > 0 {
        Html(format!(
            "<div class='bg-yellow-800 p-4 rounded text-yellow-200'>Aggregation completed with warnings<br>Bars created: {}<br>Errors: {}</div>",
            bar_count, errors
        ))
    } else {
        Html(format!(
            "<div class='bg-green-800 p-4 rounded text-green-200'>Aggregation completed successfully!<br>Bars created: {}</div>",
            bar_count
        ))
    }
}

// Batch Aggregator Form
#[derive(Deserialize)]
struct RunBatchAggregatorForm {
    market_id: Uuid,
}

async fn run_batch_aggregator_handler(
    State(state): State<AppState>,
    Form(form): Form<RunBatchAggregatorForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Running BATCH aggregator for market={}", form.market_id);

    use diesel::prelude::*;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, JoinOnDsl, BoolExpressionMethods};
    use cradle_back_end::schema::orderbooktrades;
    use cradle_back_end::schema::orderbook;
    use cradle_back_end::schema::markets::dsl as markets_dsl;

    let pool = state.config.pool.clone();
    let market_id = form.market_id;

    // Get market info to get both assets
    let market_result = {
        let pool_clone = pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool_clone.get().ok()?;
            markets_dsl::markets.find(market_id).first::<MarketRecord>(&mut conn).ok()
        }).await.unwrap()
    };

    let market = match market_result {
        Some(m) => m,
        None => return Html("<div class='text-red-400'>Market not found</div>".to_string())
    };

    let assets = vec![market.asset_one, market.asset_two];
    let now = Local::now().naive_local();

    // Define batch configurations: (duration_name, start_time, intervals)
    let batch_configs: Vec<(&str, NaiveDateTime, Vec<(&str, TimeSeriesInterval, Duration)>)> = vec![
        (
            "24 Hours",
            now - Duration::days(1),
            vec![
                ("15secs", TimeSeriesInterval::FifteenSecs, Duration::seconds(15)),
                ("30secs", TimeSeriesInterval::ThirtySecs, Duration::seconds(30)),
                ("45secs", TimeSeriesInterval::FortyFiveSecs, Duration::seconds(45)),
                ("1min", TimeSeriesInterval::OneMinute, Duration::minutes(1)),
                ("15min", TimeSeriesInterval::FifteenMinutes, Duration::minutes(15)),
                ("30min", TimeSeriesInterval::ThirtyMinutes, Duration::minutes(30)),
                ("1hr", TimeSeriesInterval::OneHour, Duration::hours(1)),
                ("4hr", TimeSeriesInterval::FourHours, Duration::hours(4)),
            ]
        ),
        (
            "7 Days",
            now - Duration::days(7),
            vec![
                ("1day", TimeSeriesInterval::OneDay, Duration::days(1)),
            ]
        ),
        (
            "30 Days",
            now - Duration::days(30),
            vec![
                ("1week", TimeSeriesInterval::OneWeek, Duration::weeks(1)),
            ]
        ),
    ];

    let mut results: Vec<String> = Vec::new();
    let mut total_bars = 0;
    let mut total_errors = 0;

    // Query all trades for the market once (for the longest duration - 30 days)
    let all_start_time = now - Duration::days(30);
    let pool_clone = pool.clone();
    let trades_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().ok()?;
        orderbooktrades::table
            .inner_join(orderbook::table.on(
                orderbooktrades::maker_order_id.eq(orderbook::id)
            ))
            .filter(
                orderbook::market_id.eq(market_id)
                    .and(orderbooktrades::created_at.ge(all_start_time))
                    .and(orderbooktrades::created_at.le(now))
            )
            .select(orderbooktrades::all_columns)
            .order_by(orderbooktrades::created_at.asc())
            .load::<OrderBookTradeRecord>(&mut conn)
            .ok()
    }).await.unwrap();

    let all_trades = match trades_result {
        Some(t) => t,
        None => return Html("<div class='text-red-400'>Failed to query trades</div>".to_string())
    };

    if all_trades.is_empty() {
        return Html("<div class='text-yellow-400'>No trades found for this market in the last 30 days</div>".to_string());
    }

    eprintln!("[ADMIN] Found {} total trades for batch processing", all_trades.len());

    // Process each batch configuration
    for (duration_name, start_time, intervals) in batch_configs {
        // Filter trades for this duration
        let duration_trades: Vec<&OrderBookTradeRecord> = all_trades
            .iter()
            .filter(|t| t.created_at >= start_time && t.created_at <= now)
            .collect();

        if duration_trades.is_empty() {
            results.push(format!("<div class='text-gray-400'>⊘ {} - No trades</div>", duration_name));
            continue;
        }

        for (interval_name, interval_enum, interval_duration) in &intervals {
            // Process each asset
            for asset_id in &assets {
                // Calculate OHLC bars
                let trades_refs: Vec<&OrderBookTradeRecord> = duration_trades.iter().copied().collect();
                let owned_trades: Vec<OrderBookTradeRecord> = trades_refs.into_iter().cloned().collect();
                let bars = calculate_ohlc_bars_for_admin(&owned_trades, start_time, now, *interval_duration);

                let mut bar_count = 0;
                let mut errors = 0;

                for (bar_start, bar_end, bar) in bars {
                    let create_input = CreateMarketTimeSeriesRecord {
                        market_id,
                        asset: *asset_id,
                        open: bar.open.clone(),
                        high: bar.high.clone(),
                        low: bar.low.clone(),
                        close: bar.close.clone(),
                        volume: bar.volume.clone(),
                        start_time: bar_start,
                        end_time: bar_end,
                        interval: Some(interval_enum.clone()),
                        data_provider_type: Some(DataProviderType::OrderBook),
                        data_provider: None,
                    };

                    let input = MarketTimeSeriesProcessorInput::AddRecord(create_input);
                    let router_input = ActionRouterInput::MarketTimeSeries(input);

                    match call_action_router(router_input, (*state.config).clone()).await {
                        Ok(_) => bar_count += 1,
                        Err(e) => {
                            eprintln!("[ADMIN] Failed to create OHLC record: {:?}", e);
                            errors += 1;
                        }
                    }
                }

                total_bars += bar_count;
                total_errors += errors;

                let status = if errors > 0 { "⚠" } else { "✓" };
                results.push(format!(
                    "<div class='text-sm'>{} {} / {} / Asset {} - {} bars</div>",
                    status, duration_name, interval_name,
                    asset_id.to_string().split('-').next().unwrap_or(""),
                    bar_count
                ));
            }
        }
    }

    eprintln!("[ADMIN] Batch aggregation complete: {} total bars, {} errors", total_bars, total_errors);

    let results_html = results.join("\n");
    let status_class = if total_errors > 0 { "bg-yellow-800 text-yellow-200" } else { "bg-green-800 text-green-200" };

    Html(format!(
        r##"<div class='{} p-4 rounded'>
            <div class='font-bold mb-2'>Batch Aggregation Complete</div>
            <div class='text-sm mb-2'>Total bars: {} | Errors: {}</div>
            <div class='border-t border-current/30 pt-2 mt-2 space-y-1 max-h-64 overflow-y-auto'>
                {}
            </div>
        </div>"##,
        status_class, total_bars, total_errors, results_html
    ))
}

// Helper struct for OHLC calculation
#[derive(Clone, Debug)]
struct OhlcBarAdmin {
    pub open: BigDecimal,
    pub high: BigDecimal,
    pub low: BigDecimal,
    pub close: BigDecimal,
    pub volume: BigDecimal,
}

fn calculate_ohlc_bars_for_admin(
    trades: &[OrderBookTradeRecord],
    start_time: NaiveDateTime,
    _end_time: NaiveDateTime,
    interval: Duration,
) -> Vec<(NaiveDateTime, NaiveDateTime, OhlcBarAdmin)> {
    if trades.is_empty() {
        return Vec::new();
    }

    let mut bars = Vec::new();
    let mut current_bucket_start = start_time;
    let mut current_bucket_trades: Vec<&OrderBookTradeRecord> = Vec::new();

    for trade in trades {
        let bucket_start = current_bucket_start;
        let bucket_end = bucket_start + interval;

        if trade.created_at >= bucket_start && trade.created_at < bucket_end {
            current_bucket_trades.push(trade);
        } else {
            if !current_bucket_trades.is_empty() {
                let bar = aggregate_trades_to_ohlc_admin(&current_bucket_trades);
                bars.push((bucket_start, bucket_end, bar));
            }

            while trade.created_at >= current_bucket_start + interval {
                current_bucket_start = current_bucket_start + interval;
            }
            current_bucket_trades = vec![trade];
        }
    }

    if !current_bucket_trades.is_empty() {
        let bucket_start = current_bucket_start;
        let bucket_end = bucket_start + interval;
        let bar = aggregate_trades_to_ohlc_admin(&current_bucket_trades);
        bars.push((bucket_start, bucket_end, bar));
    }

    bars
}

fn aggregate_trades_to_ohlc_admin(trades: &[&OrderBookTradeRecord]) -> OhlcBarAdmin {
    if trades.is_empty() {
        return OhlcBarAdmin {
            open: BigDecimal::from(0),
            high: BigDecimal::from(0),
            low: BigDecimal::from(0),
            close: BigDecimal::from(0),
            volume: BigDecimal::from(0),
        };
    }

    let mut prices = Vec::new();
    let mut volume = BigDecimal::from(0);

    for trade in trades {
        let price = if trade.maker_filled_amount != BigDecimal::from(0) {
            &trade.taker_filled_amount / &trade.maker_filled_amount
        } else {
            BigDecimal::from(0)
        };
        prices.push(price);
        volume = volume + &trade.maker_filled_amount + &trade.taker_filled_amount;
    }

    let open = prices[0].clone();
    let close = prices[prices.len() - 1].clone();
    let high = prices.iter().max().cloned().unwrap_or_else(|| BigDecimal::from(0));
    let low = prices.iter().min().cloned().unwrap_or_else(|| BigDecimal::from(0));

    OhlcBarAdmin {
        open,
        high,
        low,
        close,
        volume,
    }
}

// =============================================================================
// ADMIN ACCOUNTS TAB - Associations & KYC
// =============================================================================

async fn admin_accounts_tab_handler(State(state): State<AppState>) -> Html<String> {
    use diesel::prelude::*;
    use cradle_back_end::schema::asset_book::dsl::*;
    use cradle_back_end::schema::cradlewalletaccounts::dsl as wa_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;

    let pool = state.config.pool.clone();
    let (assets_list, wallets_list) = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get db connection");
        let assets = asset_book.load::<AssetBookRecord>(&mut conn).unwrap_or_default();
        let wallets = wa_dsl::cradlewalletaccounts.load::<CradleWalletAccountRecord>(&mut conn).unwrap_or_default();
        (assets, wallets)
    }).await.unwrap();

    Html(templates::admin_accounts_tab(assets_list, wallets_list))
}

// Form structs for association/KYC
#[derive(Deserialize, Debug)]
struct AdminAssociateForm {
    wallet_id: Option<Uuid>,       // If selecting from DB
    custom_address: Option<String>, // If entering custom address
    token_id: Uuid,
}

#[derive(Deserialize, Debug)]
struct AdminKycForm {
    wallet_id: Option<Uuid>,       // If selecting from DB
    custom_address: Option<String>, // If entering custom address
    token_id: Uuid,
}

#[derive(Deserialize, Debug)]
struct AdminAssociateAndKycForm {
    wallet_id: Option<Uuid>,
    custom_address: Option<String>,
    token_id: Uuid,
}

async fn admin_associate_token_handler(
    State(state): State<AppState>,
    Form(form): Form<AdminAssociateForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Associate token request: wallet_id={:?}, custom_address={:?}, token={}",
        form.wallet_id, form.custom_address, form.token_id);

    let pool = state.config.pool.clone();
    let mut action_wallet = state.config.wallet.clone();

    // Get the token info
    let token_result = {
        let pool_clone = pool.clone();
        let token_id = form.token_id;
        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::asset_book::dsl::*;
            use cradle_back_end::asset_book::db_types::AssetBookRecord;
            let mut conn = pool_clone.get().ok()?;
            asset_book.find(token_id).first::<AssetBookRecord>(&mut conn).ok()
        }).await.unwrap()
    };

    let token = match token_result {
        Some(t) => t,
        None => return Html("<div class='text-red-400'>Token not found</div>".to_string())
    };

    // Determine the address and contract_id to use
    let (wallet_address, contract_id, wallet_db_id) = if let Some(wallet_id) = form.wallet_id {
        // Using existing wallet from DB
        let pool_clone = pool.clone();
        let w_id = wallet_id;
        let wallet_data = tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::cradlewalletaccounts::dsl::*;
            let mut conn = pool_clone.get().ok()?;
            cradlewalletaccounts.find(w_id).first::<CradleWalletAccountRecord>(&mut conn).ok()
        }).await.unwrap();

        match wallet_data {
            Some(w) => (w.address.clone(), w.contract_id.clone(), Some(w.id)),
            None => return Html("<div class='text-red-400'>Wallet not found in database</div>".to_string())
        }
    } else if let Some(addr) = form.custom_address.filter(|s| !s.is_empty()) {
        // Using custom address - derive contract_id
        match commons::get_contract_id_from_evm_address(&addr).await {
            Ok(cid) => (addr, cid.to_string(), None),
            Err(e) => return Html(format!("<div class='text-red-400'>Failed to derive contract ID from address: {}</div>", e))
        }
    } else {
        return Html("<div class='text-red-400'>Please select a wallet or enter a custom address</div>".to_string())
    };

    eprintln!("[ADMIN] Associating token {} to address {} (contract_id: {})",
        token.symbol, wallet_address, contract_id);

    // Call the smart contract to associate the token
    let res = action_wallet
        .execute(ContractCallInput::CradleAccount(
            CradleAccountFunctionInput::AssociateToken(AssociateTokenArgs {
                token: token.token.clone(),
                account_contract_id: contract_id,
            }),
        ))
        .await;

    match res {
        Ok(ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::AssociateToken(v))) => {
            eprintln!("[ADMIN] Association tx: {:?}", v.transaction_id);

            // Update DB record if we have a wallet_db_id
            if let Some(db_id) = wallet_db_id {
                let pool_clone = pool.clone();
                let token_id = token.id;
                let _ = tokio::task::spawn_blocking(move || {
                    let mut conn = pool_clone.get().ok()?;
                    tokio::runtime::Handle::current().block_on(async {
                        update_asset_book_record(&mut conn, db_id, token_id, AssetRecordAction::Associate).await.ok()
                    })
                }).await;
            }

            Html(format!(
                "<div class='bg-green-800 p-4 rounded text-green-200'>Token associated successfully!<br>TX: {}</div>",
                v.transaction_id
            ))
        },
        Ok(_) => Html("<div class='text-red-400'>Unexpected response format</div>".to_string()),
        Err(e) => {
            eprintln!("[ADMIN] Association failed: {:?}", e);
            Html(format!("<div class='text-red-400'>Association failed: {}</div>", e))
        }
    }
}

async fn admin_grant_kyc_handler(
    State(state): State<AppState>,
    Form(form): Form<AdminKycForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Grant KYC request: wallet_id={:?}, custom_address={:?}, token={}",
        form.wallet_id, form.custom_address, form.token_id);

    let pool = state.config.pool.clone();
    let mut action_wallet = state.config.wallet.clone();

    // Get the token info
    let token_result = {
        let pool_clone = pool.clone();
        let token_id = form.token_id;
        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::asset_book::dsl::*;
            use cradle_back_end::asset_book::db_types::AssetBookRecord;
            let mut conn = pool_clone.get().ok()?;
            asset_book.find(token_id).first::<AssetBookRecord>(&mut conn).ok()
        }).await.unwrap()
    };

    let token = match token_result {
        Some(t) => t,
        None => return Html("<div class='text-red-400'>Token not found</div>".to_string())
    };

    // Check if asset has a valid asset_manager
    if !token.asset_manager.contains(".") {
        return Html("<div class='text-yellow-400'>This token does not have an asset manager that requires KYC</div>".to_string());
    }

    // Determine the address to use
    let (wallet_address, wallet_db_id) = if let Some(wallet_id) = form.wallet_id {
        // Using existing wallet from DB
        let pool_clone = pool.clone();
        let w_id = wallet_id;
        let wallet_data = tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::cradlewalletaccounts::dsl::*;
            let mut conn = pool_clone.get().ok()?;
            cradlewalletaccounts.find(w_id).first::<CradleWalletAccountRecord>(&mut conn).ok()
        }).await.unwrap();

        match wallet_data {
            Some(w) => (w.address.clone(), Some(w.id)),
            None => return Html("<div class='text-red-400'>Wallet not found in database</div>".to_string())
        }
    } else if let Some(addr) = form.custom_address.filter(|s| !s.is_empty()) {
        (addr, None)
    } else {
        return Html("<div class='text-red-400'>Please select a wallet or enter a custom address</div>".to_string())
    };

    eprintln!("[ADMIN] Granting KYC for token {} to address {}", token.symbol, wallet_address);

    // Call the smart contract to grant KYC
    let res = action_wallet
        .execute(ContractCallInput::AssetManager(
            AssetManagerFunctionInput::GrantKYC(
                token.asset_manager.clone(),
                wallet_address.clone(),
            ),
        ))
        .await;

    match res {
        Ok(ContractCallOutput::AssetManager(AssetManagerFunctionOutput::GrantKYC(v))) => {
            eprintln!("[ADMIN] KYC tx: {:?}", v.transaction_id);

            // Update DB record if we have a wallet_db_id
            if let Some(db_id) = wallet_db_id {
                let pool_clone = pool.clone();
                let token_id = token.id;
                let _ = tokio::task::spawn_blocking(move || {
                    let mut conn = pool_clone.get().ok()?;
                    tokio::runtime::Handle::current().block_on(async {
                        update_asset_book_record(&mut conn, db_id, token_id, AssetRecordAction::KYC).await.ok()
                    })
                }).await;
            }

            Html(format!(
                "<div class='bg-green-800 p-4 rounded text-green-200'>KYC granted successfully!<br>TX: {}</div>",
                v.transaction_id
            ))
        },
        Ok(_) => Html("<div class='text-red-400'>Unexpected response format</div>".to_string()),
        Err(e) => {
            eprintln!("[ADMIN] KYC grant failed: {:?}", e);
            Html(format!("<div class='text-red-400'>KYC grant failed: {}</div>", e))
        }
    }
}

async fn admin_associate_and_kyc_handler(
    State(state): State<AppState>,
    Form(form): Form<AdminAssociateAndKycForm>,
) -> Html<String> {
    eprintln!("[ADMIN] Associate & KYC request: wallet_id={:?}, custom_address={:?}, token={}",
        form.wallet_id, form.custom_address, form.token_id);

    let pool = state.config.pool.clone();
    let mut action_wallet = state.config.wallet.clone();
    let mut results = Vec::new();

    // Get the token info
    let token_result = {
        let pool_clone = pool.clone();
        let token_id = form.token_id;
        tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::asset_book::dsl::*;
            use cradle_back_end::asset_book::db_types::AssetBookRecord;
            let mut conn = pool_clone.get().ok()?;
            asset_book.find(token_id).first::<AssetBookRecord>(&mut conn).ok()
        }).await.unwrap()
    };

    let token = match token_result {
        Some(t) => t,
        None => return Html("<div class='text-red-400'>Token not found</div>".to_string())
    };

    // Determine the address and contract_id to use
    let (wallet_address, contract_id, wallet_db_id) = if let Some(wallet_id) = form.wallet_id {
        let pool_clone = pool.clone();
        let w_id = wallet_id;
        let wallet_data = tokio::task::spawn_blocking(move || {
            use diesel::prelude::*;
            use cradle_back_end::schema::cradlewalletaccounts::dsl::*;
            let mut conn = pool_clone.get().ok()?;
            cradlewalletaccounts.find(w_id).first::<CradleWalletAccountRecord>(&mut conn).ok()
        }).await.unwrap();

        match wallet_data {
            Some(w) => (w.address.clone(), w.contract_id.clone(), Some(w.id)),
            None => return Html("<div class='text-red-400'>Wallet not found in database</div>".to_string())
        }
    } else if let Some(addr) = form.custom_address.filter(|s| !s.is_empty()) {
        match commons::get_contract_id_from_evm_address(&addr).await {
            Ok(cid) => (addr, cid.to_string(), None),
            Err(e) => return Html(format!("<div class='text-red-400'>Failed to derive contract ID from address: {}</div>", e))
        }
    } else {
        return Html("<div class='text-red-400'>Please select a wallet or enter a custom address</div>".to_string())
    };

    // Step 1: Associate token
    eprintln!("[ADMIN] Step 1: Associating token {} to address {}", token.symbol, wallet_address);
    let assoc_res = action_wallet
        .execute(ContractCallInput::CradleAccount(
            CradleAccountFunctionInput::AssociateToken(AssociateTokenArgs {
                token: token.token.clone(),
                account_contract_id: contract_id.clone(),
            }),
        ))
        .await;

    match assoc_res {
        Ok(ContractCallOutput::CradleAccount(CradleAccountFunctionOutput::AssociateToken(v))) => {
            results.push(format!("✓ Association TX: {}", v.transaction_id));

            if let Some(db_id) = wallet_db_id {
                let pool_clone = pool.clone();
                let token_id = token.id;
                let _ = tokio::task::spawn_blocking(move || {
                    let mut conn = pool_clone.get().ok()?;
                    tokio::runtime::Handle::current().block_on(async {
                        update_asset_book_record(&mut conn, db_id, token_id, AssetRecordAction::Associate).await.ok()
                    })
                }).await;
            }
        },
        Ok(_) => results.push("✗ Association: Unexpected response".to_string()),
        Err(e) => results.push(format!("✗ Association failed: {}", e)),
    }

    // Step 2: Grant KYC (if asset manager exists)
    if token.asset_manager.contains(".") {
        eprintln!("[ADMIN] Step 2: Granting KYC for token {} to address {}", token.symbol, wallet_address);
        let kyc_res = action_wallet
            .execute(ContractCallInput::AssetManager(
                AssetManagerFunctionInput::GrantKYC(
                    token.asset_manager.clone(),
                    wallet_address.clone(),
                ),
            ))
            .await;

        match kyc_res {
            Ok(ContractCallOutput::AssetManager(AssetManagerFunctionOutput::GrantKYC(v))) => {
                results.push(format!("✓ KYC TX: {}", v.transaction_id));

                if let Some(db_id) = wallet_db_id {
                    let pool_clone = pool.clone();
                    let token_id = token.id;
                    let _ = tokio::task::spawn_blocking(move || {
                        let mut conn = pool_clone.get().ok()?;
                        tokio::runtime::Handle::current().block_on(async {
                            update_asset_book_record(&mut conn, db_id, token_id, AssetRecordAction::KYC).await.ok()
                        })
                    }).await;
                }
            },
            Ok(_) => results.push("✗ KYC: Unexpected response".to_string()),
            Err(e) => results.push(format!("✗ KYC failed: {}", e)),
        }
    } else {
        results.push("⊘ KYC: Skipped (no asset manager)".to_string());
    }

    let has_errors = results.iter().any(|r| r.starts_with("✗"));
    let result_html = results.join("<br>");

    if has_errors {
        Html(format!(
            "<div class='bg-yellow-800 p-4 rounded text-yellow-200'>Completed with errors:<br>{}</div>",
            result_html
        ))
    } else {
        Html(format!(
            "<div class='bg-green-800 p-4 rounded text-green-200'>All operations successful!<br>{}</div>",
            result_html
        ))
    }
}

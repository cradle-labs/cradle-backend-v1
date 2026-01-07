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

use cradle_back_end::utils::app_config::AppConfig;
use cradle_back_end::accounts::db_types::{CradleWalletAccountRecord, CreateCradleAccount, CradleAccountType, CradleAccountStatus};
use cradle_back_end::market::processor_enums::MarketProcessorInput;
use cradle_back_end::market::db_types::MarketRecord;
use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::cli_helper::call_action_router;

// Ops for Faucet/OnRamp
use cradle_back_end::ramper::{Ramper, OnRampRequest};
use cradle_back_end::accounts::operations::{associate_token, kyc_token};
use cradle_back_end::accounts::processor_enums::{AssociateTokenToWalletInputArgs, GrantKYCInputArgs};
use cradle_back_end::asset_book::operations::{get_asset, get_wallet, mint_asset};
use contract_integrator::utils::functions::{
    ContractCallInput,
    asset_manager::{AirdropArgs, AssetManagerFunctionInput},
    commons::{ContractFunctionProcessor, get_account_balances},
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

// Re-add existing Place Order Handler
#[derive(Deserialize, Debug)]
struct PlaceOrderForm {
    account_id: Uuid,
    market_id: Uuid,
    side: String,       // "buy" or "sell"
    order_type: String, // "limit" or "market"
    price: Option<String>,
    amount: String,
}

async fn place_order_handler(
    State(state): State<AppState>,
    Form(form): Form<PlaceOrderForm>,
) -> Html<String> {
    eprintln!("[DEBUG] Place order request: account_id={}, market_id={}, side={}, type={}, amount={}, price={:?}", 
        form.account_id, form.market_id, form.side, form.order_type, form.amount, form.price);
    
    // Fetch Market
    let input = MarketProcessorInput::GetMarket(form.market_id);
    let router_input = ActionRouterInput::Markets(input);
    let market_record = match call_action_router(router_input, (*state.config).clone()).await {
        Ok(ActionRouterOutput::Markets(cradle_back_end::market::processor_enums::MarketProcessorOutput::GetMarket(m))) => m,
        _ => return Html("<tr><td colspan='5' class='text-red-500'>Market not found</td></tr>".to_string())
    };

    let (bid_asset_id, ask_asset_id) = if form.side == "sell" {
        (market_record.asset_two, market_record.asset_one)
    } else {
        (market_record.asset_one, market_record.asset_two)
    };
    
    // Fetch asset records to get decimals
    use cradle_back_end::schema::asset_book::dsl as ab_dsl;
    use cradle_back_end::asset_book::db_types::AssetBookRecord;
    use diesel::prelude::*;
    
    let pool = state.config.pool.clone();
    let bid_asset_id_copy = bid_asset_id;
    let ask_asset_id_copy = ask_asset_id;
    
    let assets_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().ok()?;
        let bid_asset = ab_dsl::asset_book
            .find(bid_asset_id_copy)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        let ask_asset = ab_dsl::asset_book
            .find(ask_asset_id_copy)
            .first::<AssetBookRecord>(&mut conn)
            .ok()?;
        Some((bid_asset, ask_asset))
    }).await.unwrap();
    
    let (bid_asset, ask_asset) = match assets_result {
        Some(assets) => assets,
        None => return Html("<tr><td colspan='5' class='text-red-500'>Failed to fetch asset details</td></tr>".to_string())
    };
    
    eprintln!("[DEBUG] Bid asset: {} (decimals: {}), Ask asset: {} (decimals: {})", 
        bid_asset.symbol, bid_asset.decimals, ask_asset.symbol, ask_asset.decimals);
    
    let amount = BigDecimal::from_str(&form.amount).unwrap_or(BigDecimal::from(0));
    let price = form.price.as_ref().map(|p| BigDecimal::from_str(p).unwrap_or(BigDecimal::from(0))).unwrap_or(BigDecimal::from(0));
    
    // Calculate bid and ask amounts with proper decimal scaling
    // Price is in bid asset decimals
    let bid_multiplier = BigDecimal::from(10i64.pow(bid_asset.decimals as u32));
    let ask_multiplier = BigDecimal::from(10i64.pow(ask_asset.decimals as u32));
    
    let (bid_amt, ask_amt) = if form.side == "buy" {
        // Buying: bid_amt = amount * price (both in bid decimals), ask_amt = amount (in ask decimals)
        (
            (amount.clone() * ask_multiplier),
            (amount.clone() * price.clone() * bid_multiplier.clone())
        )
    } else {
        // Selling: bid_amt = amount (in bid decimals), ask_amt = amount * price (price in bid decimals, convert to ask)
        (
            (amount.clone() * price.clone() * bid_multiplier.clone()),
            (amount.clone() * ask_multiplier)
        )
    };
    
    eprintln!("[DEBUG] Calculated amounts - bid_amt: {}, ask_amt: {}", bid_amt, ask_amt);

    use cradle_back_end::order_book::processor_enums::OrderBookProcessorInput;
    use cradle_back_end::order_book::db_types::{NewOrderBookRecord, OrderType as DbOrderType, FillMode};
    
    let o_type = match form.order_type.as_str() {
        "market" => DbOrderType::Market,
        _ => DbOrderType::Limit
    };

    let new_order = NewOrderBookRecord {
        wallet: form.account_id,
        market_id: form.market_id,
        bid_asset: bid_asset_id,
        ask_asset: ask_asset_id,
        bid_amount: bid_amt,
        ask_amount: ask_amt,
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
            Html(r#"<tr class="bg-green-900/40"><td colspan="5" class="p-3 text-center text-green-300">Order Submitted! Refreshing...</td></tr>"#.to_string())
        },
        Err(e) => {
            eprintln!("[ERROR] Order submission failed: {:?}", e);
            Html(format!(r#"<tr class="bg-red-900/40"><td colspan="5" class="p-3 text-center text-red-300">Error: {}</td></tr>"#, e))
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

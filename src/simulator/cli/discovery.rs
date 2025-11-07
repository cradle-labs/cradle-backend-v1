use anyhow::{Result, anyhow};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use uuid::Uuid;

use crate::accounts::db_types::{CradleAccountRecord, CradleWalletAccountRecord};
use crate::market::db_types::MarketRecord;
use crate::schema::{cradleaccounts, markets, asset_book, cradlewalletaccounts};
use crate::simulator::budget::storage::BudgetStore;

pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

/// Discover all test accounts (linked_account_id starts with prefix)
pub fn discover_accounts(
    conn: &mut DbConn,
    account_prefix: &str,
) -> Result<Vec<CradleAccountRecord>> {
    use crate::schema::cradleaccounts::dsl::*;
    use chrono::NaiveDate;
    
    let cutoff_date = NaiveDate::from_ymd_opt(2025, 11, 3)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let accounts = cradleaccounts
        .filter(linked_account_id.like(format!("{}%", account_prefix)))
        .load::<CradleAccountRecord>(conn)?;

    Ok(accounts)
}

/// Discover all available markets
pub fn discover_markets(conn: &mut DbConn) -> Result<Vec<MarketRecord>> {
    use crate::schema::markets::dsl::*;

    let markets_list = markets
        .load::<MarketRecord>(conn)?;

    Ok(markets_list)
}

/// Get wallet for a specific account
pub fn get_wallet_for_account(
    conn: &mut DbConn,
    account_id: Uuid,
) -> Result<CradleWalletAccountRecord> {
    use crate::schema::cradlewalletaccounts::dsl::*;

    cradlewalletaccounts
        .filter(cradle_account_id.eq(account_id))
        .first::<CradleWalletAccountRecord>(conn)
        .map_err(|e| anyhow!("Failed to find wallet for account {}: {}", account_id, e))
}

/// Get all unique assets from markets
pub fn get_market_assets(conn: &mut DbConn) -> Result<Vec<Uuid>> {
    let markets_list = discover_markets(conn)?;

    let mut assets = Vec::new();
    for market in markets_list {
        if !assets.contains(&market.asset_one) {
            assets.push(market.asset_one);
        }
        if !assets.contains(&market.asset_two) {
            assets.push(market.asset_two);
        }
    }

    Ok(assets)
}

/// Initialize budgets for all accounts and assets
pub fn initialize_budgets(
    conn: &mut DbConn,
    budget_store: &mut BudgetStore,
    accounts: &[CradleAccountRecord],
    initial_budget: BigDecimal,
) -> Result<()> {
    let assets = get_market_assets(conn)?;

    let mut initialized = 0;

    for account in accounts {
        for asset in &assets {
            match budget_store.set_budget(account.id, *asset, initial_budget.clone()) {
                Ok(_) => initialized += 1,
                Err(e) => {
                    eprintln!("Warning: Failed to initialize budget for account {} asset {}: {}",
                        account.id, asset, e);
                }
            }
        }
    }

    println!(
        "Initialized {} budgets ({}x{} = {} per account/asset)",
        initialized,
        accounts.len(),
        assets.len(),
        initial_budget
    );

    Ok(())
}


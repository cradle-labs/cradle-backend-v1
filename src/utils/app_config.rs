use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use anyhow::Result;
use contract_integrator::wallet::wallet::ActionWallet;
use dotenvy::dotenv;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
    pub wallet: ActionWallet
}

impl AppConfig {
    pub fn new(pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>, wallet: ActionWallet)-> Self {
        Self {
            pool,
            wallet
        }
    }

    pub fn from_env()->Result<Self>{
        dotenv()?;

        let DATABASE_URL = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env file or environment variables");
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
        let pool = Pool::new(manager)?;

        let wallet = ActionWallet::from_env();

        Ok(Self::new(pool, wallet))
    }
}
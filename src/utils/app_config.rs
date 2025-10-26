use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use anyhow::Result;
use contract_integrator::wallet::wallet::ActionWallet;

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

        todo!("Implement app config from env")
    }
}
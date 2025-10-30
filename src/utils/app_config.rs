use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use anyhow::{anyhow, Result};
use contract_integrator::wallet::wallet::ActionWallet;
use dotenvy::dotenv;
use socketioxide::SocketIo;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
    pub wallet: ActionWallet,
    io: Option<SocketIo>
}

impl AppConfig {
    pub fn new(pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>, wallet: ActionWallet)-> Self {
        Self {
            pool,
            wallet,
            io: None
        }
    }

    pub fn from_env()->Result<Self>{
        let _ = dotenv();

        let DATABASE_URL = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env file or environment variables");
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
        let pool = Pool::new(manager)?;

        let wallet = ActionWallet::from_env();

        Ok(Self::new(pool, wallet))
    }

    pub fn set_io(&mut self, io: SocketIo){
        self.io = Some(io);
    }

    pub fn get_io(&self)->Result<SocketIo> {
        self.io.clone().ok_or_else(||anyhow!("Failed to get socket io"))
    }
}
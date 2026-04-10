use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use anyhow::{anyhow, Result};
use contract_integrator::wallet::wallet::ActionWallet;
use dotenvy::dotenv;
use socketioxide::SocketIo;
use crate::utils::cache::RedisPool;

#[derive(Clone)]
pub struct AppConfig {
    pub pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
    pub wallet: ActionWallet,
    pub redis: Option<RedisPool>,
    io: Option<SocketIo>
}

impl std::fmt::Debug for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppConfig")
            .field("pool", &self.pool)
            .field("wallet", &self.wallet)
            .field("redis", &self.redis.as_ref().map(|_| "RedisPool(connected)"))
            .field("io", &self.io)
            .finish()
    }
}

impl AppConfig {
    pub fn new(pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>, wallet: ActionWallet)-> Self {
        Self {
            pool,
            wallet,
            redis: None,
            io: None
        }
    }

    pub fn from_env()->Result<Self>{
        let _ = dotenv();

        let DATABASE_URL = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env file or environment variables");
        let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
        let pool = Pool::builder()
            .max_size(50)
            .min_idle(Some(5))
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)?;

        let wallet = ActionWallet::from_env();

        Ok(Self::new(pool, wallet))
    }

    pub fn set_io(&mut self, io: SocketIo){
        self.io = Some(io);
    }

    pub fn get_io(&self)->Result<SocketIo> {
        self.io.clone().ok_or_else(||anyhow!("Failed to get socket io"))
    }

    pub fn set_redis(&mut self, redis: RedisPool) {
        self.redis = Some(redis);
    }
}
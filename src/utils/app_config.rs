use diesel::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>
}

impl AppConfig {
    pub fn new(pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>)-> Self {
        Self {
            pool
        }
    }

    pub fn from_env()->Result<Self>{

        todo!("Implement app config from env")
    }
}
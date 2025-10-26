use anyhow::Result;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use crate::action_router::ActionRouterInput;
use crate::utils::app_config::AppConfig;

pub trait ActionProcessor<Config, Output> {
    async fn process(&self, app_config: &mut AppConfig, local_config: &mut Config, conn: Option<&mut PooledConnection<ConnectionManager<PgConnection>>>)->Result<Output>;
}
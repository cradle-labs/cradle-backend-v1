use anyhow::anyhow;
use bigdecimal::BigDecimal;
use contract_integrator::wallet::wallet::ActionWallet;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, PooledConnection},
};
use std::env;

pub struct SystemAddresses {
    pub fee_collector: String,
}
pub fn get_system_addresses() -> SystemAddresses {
    SystemAddresses {
        fee_collector: env::var("FEE_COLLECTOR").unwrap(),
    }
}

pub type DbConn<'db> = &'db mut PooledConnection<ConnectionManager<PgConnection>>;
pub type TaskWallet<'wt> = &'wt mut ActionWallet;

#[macro_export]
macro_rules! extract_option {
    ($option: expr) => {{ $option.ok_or_else(|| anyhow!("failed to extract option")) }};
}

#[macro_export]
macro_rules! big_to_u64 {
    ($num: expr) => {{ $crate::extract_option!($num.to_u64()) }};
}

#[macro_export]
macro_rules! map_to_api_error {
    ($call: expr, $msg: literal) => {{ $call.map_err(|_| ApiError::InternalError(String::from($msg))) }};
}

#[macro_export]
macro_rules! address_to_id {
    ($address: expr) => {{ contract_integrator::utils::functions::commons::get_contract_id_from_evm_address($address) }};
}

#[macro_export]
macro_rules! collect_input {
    ($p: literal, $t: ty) => {{
        let incoming: $t = Input::new().with_prompt($p).interact()?;
        incoming
    }};
    ($p: literal, $default: expr, $t: ty) => {{
        let incoming: $t = Input::new().default($default).with_prompt($p).interact()?;
        incoming
    }};
}

#[macro_export]
macro_rules! choose {
    ($p: literal, $($item: literal),+ )=> {{
        let items = vec![$($item),+];
        let selected = Select::new().with_prompt($p).items(&items).interact()?;

        selected
    }}
}

#[macro_export]
macro_rules! perr {
    ($err: expr) => {{
        print_error(&format!("ERROR:: {:?}", $err));
    }};
}

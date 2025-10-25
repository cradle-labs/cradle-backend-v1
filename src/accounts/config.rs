use contract_integrator::wallet::wallet::ActionWallet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct AccountProcessorConfig {
// TODO: add account specific env variables
    pub wallet: ActionWallet
}
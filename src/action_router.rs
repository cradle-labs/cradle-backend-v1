use crate::accounts::processor_enums::{AccountsProcessorInput, AccountsProcessorOutput};
use crate::utils::app_config::AppConfig;
use anyhow::Result;
use contract_integrator::wallet::wallet::ActionWallet;
use crate::accounts::config::AccountProcessorConfig;
use crate::utils::db::get_conn;
use crate::utils::traits::ActionProcessor;

pub enum ActionRouterInput {
    Accounts(AccountsProcessorInput)
}

pub enum ActionRouterOutput {
    Accounts(AccountsProcessorOutput)
}


impl ActionRouterInput {

    pub async fn process(&self, app_config: AppConfig)-> Result<ActionRouterOutput> {
        match self {
            ActionRouterInput::Accounts(processor) => {
                let mut conn = get_conn(app_config.pool.clone())?;
                // TODO: possibility of filtering out so conn's only available to necessary processors, future optimization
                let wallet = ActionWallet::from_env();
                let mut processor_config = AccountProcessorConfig {
                    wallet
                };
                let res = processor.process(app_config.clone(), &mut processor_config, Some(&mut conn)).await?;
                Ok(ActionRouterOutput::Accounts(res))
            }
        }
    }
}

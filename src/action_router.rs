use crate::accounts::processor_enums::{AccountsProcessorInput, AccountsProcessorOutput};
use crate::utils::app_config::AppConfig;
use anyhow::Result;
use contract_integrator::wallet::wallet::ActionWallet;
use crate::accounts::config::AccountProcessorConfig;
use crate::asset_book::config::AssetBookConfig;
use crate::asset_book::processor_enums::{AssetBookProcessorInput, AssetBookProcessorOutput};
use crate::market::processor_enums::{MarketProcessorInput, MarketProcessorOutput};
use crate::market_time_series::config::MarketTimeSeriesConfig;
use crate::market_time_series::processor_enum::{MarketTimeSeriesProcessorInput, MarketTimeSeriesProcessorOutput};
use crate::utils::db::get_conn;
use crate::utils::traits::ActionProcessor;

pub enum ActionRouterInput {
    Accounts(AccountsProcessorInput),
    AssetBook(AssetBookProcessorInput),
    Markets(MarketProcessorInput),
    MarketTimeSeries(MarketTimeSeriesProcessorInput)
}

pub enum ActionRouterOutput {
    Accounts(AccountsProcessorOutput),
    AssetBook(AssetBookProcessorOutput),
    Markets(MarketProcessorOutput),
    MarketTimeSeries(MarketTimeSeriesProcessorOutput)
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
                let res = processor.process(&mut app_config.clone(), &mut processor_config, Some(&mut conn)).await?;
                Ok(ActionRouterOutput::Accounts(res))
            }
            ActionRouterInput::AssetBook(processor)=>{

                let mut conn = get_conn(app_config.pool.clone())?;

                let mut config = AssetBookConfig {

                };

                let res = processor.process(&mut app_config.clone(), &mut config, Some(&mut conn)).await?;

                Ok(ActionRouterOutput::AssetBook(res))
            },
            ActionRouterInput::Markets(processor)=>{
                let mut conn = get_conn(app_config.pool.clone())?;

                let mut config = crate::market::config::MarketsConfig {

                };

                let res = processor.process(&mut app_config.clone(), &mut config, Some(&mut conn)).await?;

                Ok(ActionRouterOutput::Markets(res))
            },
            ActionRouterInput::MarketTimeSeries(processor)=>{
                let mut conn = get_conn(app_config.pool.clone())?;

                let mut config = MarketTimeSeriesConfig {

                };

                let res = processor.process(&mut app_config.clone(), &mut config, Some(&mut conn)).await?;

                Ok(ActionRouterOutput::MarketTimeSeries(res))
            }
        }
    }
}

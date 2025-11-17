use crate::listing::config::CradleNativeListingsConfig;
use crate::listing::operations::*;
use crate::{
    listing::processor_enums::{
        CradleNativeListingFunctionsInput, CradleNativeListingFunctionsOutput,
    },
    utils::traits::ActionProcessor,
};
use anyhow::{Result, anyhow};

impl ActionProcessor<CradleNativeListingsConfig, CradleNativeListingFunctionsOutput>
    for CradleNativeListingFunctionsInput
{
    async fn process(
        &self,
        app_config: &mut crate::utils::app_config::AppConfig,
        local_config: &mut CradleNativeListingsConfig,
        conn: Option<
            &mut diesel::r2d2::PooledConnection<
                diesel::r2d2::ConnectionManager<diesel::PgConnection>,
            >,
        >,
    ) -> anyhow::Result<CradleNativeListingFunctionsOutput> {
        let app_conn = conn.ok_or_else(|| anyhow!("Unable to retrieve conn"))?;
        let mut wallet = app_config.wallet.clone();
        match self {
            CradleNativeListingFunctionsInput::CreateCompany(input) => {
                let res = create_company(app_conn, &mut wallet, input.clone()).await?;
                Ok(CradleNativeListingFunctionsOutput::CreateCompany(res))
            }
            CradleNativeListingFunctionsInput::CreateListing(input) => {
                let res = create_listing(app_conn, &mut wallet, input.clone()).await?;
                Ok(CradleNativeListingFunctionsOutput::CreateListing(res))
            }
            CradleNativeListingFunctionsInput::Purchase(input) => {
                purchase(app_conn, &mut wallet, input.clone()).await?;
                Ok(CradleNativeListingFunctionsOutput::Purchase)
            }
            CradleNativeListingFunctionsInput::ReturnAsset(input) => {
                return_asset(app_conn, &mut wallet, input.clone());
                Ok(CradleNativeListingFunctionsOutput::ReturnAsset)
            }
            CradleNativeListingFunctionsInput::WithdrawToBeneficiary(input) => {
                withdraw_to_beneficiary(app_conn, &mut wallet, input.clone());
                Ok(CradleNativeListingFunctionsOutput::WithdrawToBeneficiary)
            }
            CradleNativeListingFunctionsInput::GetStats(input) => {
                let res = get_listing_stats(app_conn, &mut wallet, *input).await?;
                Ok(CradleNativeListingFunctionsOutput::GetStats(res))
            }
            CradleNativeListingFunctionsInput::GetFee(input) => {
                let res = get_purchase_fee(app_conn, &mut wallet, input.clone()).await?;
                Ok(CradleNativeListingFunctionsOutput::GetFee(res))
            }
        }
    }
}

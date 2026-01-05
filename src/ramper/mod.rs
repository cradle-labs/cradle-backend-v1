use crate::{
    accounts::{operations::associate_token, processor_enums::AssociateTokenToWalletInputArgs},
    asset_book::operations::{get_asset, get_wallet},
    big_to_u64,
    utils::commons::{DbConn, TaskWallet},
};
use anyhow::{Result, anyhow};
use bigdecimal::{BigDecimal, ToPrimitive};
use clap::{Parser, ValueEnum};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::instrument::WithSubscriber;
use uuid::Uuid;

#[derive(Parser, Deserialize, Serialize, Clone)]
pub struct Ramper {
    #[clap(long, env)]
    pub ramper_token: String,
    #[clap(long, env)]
    pub ramper_webhook_secret: String,
    #[clap(long, env)]
    pub ramper_callback: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OnRampRequest {
    pub token: Uuid,
    pub amount: BigDecimal,
    pub wallet_id: Uuid,
    pub result_page: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OnRampResponse {
    pub reference: String,
    pub authorization_url: String,
    pub access_code: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestMetadata {
    #[serde(rename = "orderID")]
    pub order_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct RequestToken {
    pub token: String,
    pub amount: u64,
    pub email: String,
    pub currency: String,
    pub metadata: RequestMetadata,
    pub callback_url: String,
    pub channels: Vec<String>,
    pub crypto_account: String,
}

// {
//   event_type: string;
//   order_id: <orderid-given-when-initializing-payment>;
//   token: <token-you-indicated-to-receive>;
//   amount: <amount-user-paid>;
//   currency: <currency-user-pay-in> | null;
//   failureReason?: <optional-further-explanation-incase-failure>;
// }
#[derive(Serialize, Deserialize)]
pub struct CallbackData {
    pub event_type: String,
    pub order_id: String,
    pub token: String,
    pub amount: String,
    pub currency: Option<String>,
    #[serde(rename = "failureReason")]
    pub failure_reason: Option<String>,
}

impl Ramper {
    pub fn from_env() -> Result<Self> {
        Self::try_parse().map_err(|e| {
            println!("Fetch Ramper errror {:?}", e);
            anyhow!(e)
        })
    }

    pub async fn onramp<'a>(
        &self,
        wallet: TaskWallet<'a>,
        conn: DbConn<'a>,
        req: OnRampRequest,
    ) -> Result<OnRampResponse> {
        associate_token(
            conn,
            wallet,
            AssociateTokenToWalletInputArgs {
                wallet_id: req.wallet_id,
                token: req.token,
            },
        )
        .await?;

        let token = get_asset(conn, req.token).await?;
        let wallet_data = get_wallet(conn, req.wallet_id).await?;
        let order_id = Uuid::new_v4().to_string();

        let ramp_request = RequestToken {
            token: token.symbol,
            amount: big_to_u64!(req.amount)?,
            email: req.email,
            currency: "KES".to_string(),
            metadata: RequestMetadata { order_id },
            callback_url: req.result_page,
            channels: vec!["card".to_string()],
            crypto_account: wallet_data.contract_id,
        };

        let client = Client::new();

        let response = client
            .post("https://test.api.orionramp.com/api/transaction/initialize")
            .header(
                "Authorization",
                format!("Bearer {}", self.ramper_token.clone()),
            )
            .header("Content-Type", "application/json")
            .json(&ramp_request)
            .send()
            .await?;

        let result = response.json::<OnRampResponse>().await?;

        Ok(result)
    }

    pub async fn callback_handler<'a>(
        &self,
        conn: DbConn<'a>,
        callback: CallbackData,
    ) -> Result<()> {
        //
        Ok(())
    }
}

use crate::{
    api::{error::ApiError, response::ApiResponse},
    listing::{
        db_types::{CradleNativeListingRow, ListingStatus},
        operations::get_listing,
    },
    utils::app_config::AppConfig,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use diesel::QueryDsl;
use diesel::prelude::*;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument::WithSubscriber;
use uuid::Uuid;

// /listings/{id}
pub async fn get_listing_by_id(
    State(app_config): State<AppConfig>,
    Path(listing_id): Path<Uuid>,
) -> Result<(StatusCode, Json<ApiResponse<CradleNativeListingRow>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::DatabaseError("Failed to connect".to_string()))?;
    match get_listing(&mut conn, listing_id).await {
        Ok(v) => Ok((
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(v),
                error: None,
            }),
        )),
        Err(_) => Err(ApiError::NotFound("Listing not found".to_string())),
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListingQueryParams {
    pub company: Option<Uuid>,
    pub listed_asset: Option<Uuid>,
    pub purchase_asset: Option<Uuid>,
    pub status: Option<ListingStatus>,
}

// /listings
pub async fn get_listings(
    State(app_config): State<AppConfig>,
    Query(params): Query<ListingQueryParams>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<CradleNativeListingRow>>>), ApiError> {
    let mut conn = app_config
        .pool
        .get()
        .map_err(|_| ApiError::DatabaseError("Failed to connect".to_string()))?;

    match {
        use crate::schema::cradlenativelistings::dsl::*;
        use crate::schema::cradlenativelistings::*;

        let mut query = cradlenativelistings.filter(id.is_not_null()).into_boxed();

        if let Some(company_value) = &params.company {
            query = query.filter(company.eq(company_value));
        };

        if let Some(value) = &params.purchase_asset {
            query = query.filter(purchase_with_asset.eq(value));
        };

        if let Some(value) = &params.status {
            query = query.filter(status.eq(value));
        };

        if let Some(value) = &params.listed_asset {
            query = query.filter(listed_asset.eq(value));
        };

        query.get_results::<CradleNativeListingRow>(&mut conn)
    } {
        Ok(results) => Ok((
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                data: Some(results),
                error: None,
            }),
        )),
        Err(_) => Err(ApiError::DatabaseError("".to_string())),
    }
}

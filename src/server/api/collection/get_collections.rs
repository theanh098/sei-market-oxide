use crate::{
    database::repository::collection,
    error::AppError,
    server::{
        deserialization::SortDirection,
        extract::{
            state::Postgres,
            validate::{self, ValidatedQuery},
        },
        serialization::{PaginatedData, SerializedResponse},
    },
};
use axum::{extract::Query, Json};
use serde::Deserialize;
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Deserialize, IntoParams, Validate)]
#[into_params(parameter_in = Query)]
pub struct Params {
    #[validate(range(min = 1))]
    pub limit: u8,
    pub page: u64,
    pub search: Option<String>,
    pub sort_by: SortBy,
    pub sort_direction: SortDirection,
}

#[derive(Deserialize, ToSchema, Debug)]
pub enum SortBy {
    #[serde(rename = "1h")]
    _1h,

    #[serde(rename = "24h")]
    _24h,

    #[serde(rename = "7d")]
    _7d,

    #[serde(rename = "30d")]
    _30d,

    #[serde(rename = "all")]
    All,
}

#[utoipa::path(
  get,
  params(
    Params
  ),
  path = "/api/v1/collections",
  tag = "Collection",
  responses(
      (status = 200, description = "return list collections")
  ),
  security(
    ("BearerAuth" = []),
  )
)]
pub async fn get_collections(
    ValidatedQuery(Params {
        limit,
        page,
        search,
        sort_by,
        sort_direction,
    }): ValidatedQuery<Params>,
    Postgres(db): Postgres,
) -> Result<Json<Value>, AppError> {
    let (collections, total) =
        collection::find_collections_with_stats(&db, search, page, limit, sort_by, sort_direction)
            .await?;

    let data = PaginatedData {
        nodes: collections,
        page,
        total,
    };

    data.into_response()
}

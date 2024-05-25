use crate::server::extract::{
    state::Postgres,
    validate::{self, ValidatedQuery},
};
use axum::extract::Query;
use serde::Deserialize;
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

#[derive(Deserialize, ToSchema)]
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

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Desc,
    Asc,
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
    Postgres(_db): Postgres,
) {
    println!("{}", limit)
}

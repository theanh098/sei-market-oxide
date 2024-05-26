use crate::{
    database::repository::{self},
    error::AppError,
    server::{
        extract::{
            state::Postgres,
        },
    },
};

use chrono::{Months, Utc};






#[utoipa::path(
  get,
  path = "/api/v1/leaderboard",
  tag = "Leaderboard",
  responses(
      (status = 200, description = "return leaderboard")
  )
)]
pub async fn get_leaderboad(Postgres(db): Postgres) -> Result<(), AppError> {
    let from = Utc::now()
        .checked_sub_months(Months::new(10))
        .unwrap_or_default();

    let to = Utc::now();

    repository::user_point::find_leaderboad_by_date(
        &db,
        from,
        to,
        1,
        20,
        Some("sei1932egdcxujcgg6r7fgpef4xj9c6glm8tyz8tpd".to_owned()),
    )
    .await?;
    Ok(())
}

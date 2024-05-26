use sea_orm::Order;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Desc,
    Asc,
}
impl SortDirection {
    pub fn into_order(self) -> Order {
        match self {
            Self::Asc => Order::Asc,
            Self::Desc => Order::Desc,
        }
    }
}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "collection_view")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub image: Option<String>,
    pub banner: Option<String>,
    pub description: Option<String>,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))", nullable)]
    pub royalty: Option<Decimal>,

    pub supply: i32,

    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub socials: Option<Json>,

    pub listed: i64,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub floor_price: Decimal,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub ceiling_price: Decimal,

    pub sales: i64,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub volume: Decimal,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub volume_of_1h: Decimal,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub volume_of_24h: Decimal,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub volume_of_7d: Decimal,

    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub volume_of_30d: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

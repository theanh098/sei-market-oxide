//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use super::sea_orm_active_enums::Marketplace;
use super::sea_orm_active_enums::NftActivityKind;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "nft_activity")]
pub struct Model {
    pub tx_hash: String,
    pub seller_address: Option<String>,
    pub buyer_address: Option<String>,
    pub date: DateTimeWithTimeZone,
    #[sea_orm(column_type = "Decimal(Some((90, 2)))")]
    pub price: Decimal,
    pub denom: String,
    pub event_kind: NftActivityKind,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Json,
    pub nft_id: i32,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub market: Marketplace,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::nft::Entity",
        from = "Column::NftId",
        to = "super::nft::Column::Id",
        on_update = "Cascade",
        on_delete = "Restrict"
    )]
    Nft,
}

impl Related<super::nft::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nft.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

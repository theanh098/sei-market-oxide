use crate::database::entity::{
    nft_activity,
    prelude::NftActivity,
    sea_orm_active_enums::{Marketplace, NftActivityKind},
};
use sea_orm::{
    prelude::{DateTimeUtc, Decimal},
    DatabaseTransaction, DbErr, EntityTrait, Set,
};

pub async fn create(
    tx: &DatabaseTransaction,
    params: CreateNftActivityParams,
) -> Result<(), DbErr> {
    let activity = nft_activity::ActiveModel {
        denom: Set(params.denom),
        buyer_address: Set(params.buyer_address),
        date: Set(params.created_date.into()),
        event_kind: Set(params.event_kind),
        market: Set(params.marketplace),
        metadata: Set(params.metadata),
        nft_id: Set(params.nft_id),
        price: Set(params.price),
        seller_address: Set(params.seller_address),
        tx_hash: Set(params.tx_hash),
        ..Default::default()
    };

    NftActivity::insert(activity).exec(tx).await?;

    Ok(())
}

pub struct CreateNftActivityParams {
    pub denom: String,
    pub metadata: serde_json::Value,
    pub price: Decimal,
    pub event_kind: NftActivityKind,
    pub nft_id: i32,
    pub tx_hash: String,
    pub seller_address: Option<String>,
    pub buyer_address: Option<String>,
    pub created_date: DateTimeUtc,
    pub marketplace: Marketplace,
}

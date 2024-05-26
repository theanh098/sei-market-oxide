use sea_orm::{
    prelude::{DateTimeUtc, Decimal},
    DatabaseTransaction, DbErr, EntityTrait, Set,
};

use crate::database::entity::{sea_orm_active_enums::Marketplace, transaction};

pub async fn create(
    tx: &DatabaseTransaction,
    params: CreateTransactionParams,
) -> Result<(), DbErr> {
    let transaction = transaction::ActiveModel {
        buyer_address: Set(params.buyer_address),
        collection_address: Set(params.collection_address),
        date: Set(params.created_date.into()),
        market: Set(params.marketplace),
        seller_address: Set(params.seller_address),
        txn_hash: Set(params.tx_hash),
        volume: Set(params.volume),
        ..Default::default()
    };

    transaction::Entity::insert(transaction).exec(tx).await?;

    Ok(())
}

pub struct CreateTransactionParams {
    pub tx_hash: String,
    pub volume: Decimal,
    pub collection_address: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub created_date: DateTimeUtc,
    pub marketplace: Marketplace,
}

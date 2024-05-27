use crate::database::entity::{sea_orm_active_enums::StreamContext, stream_tx};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};

#[allow(dead_code)]
pub async fn find_stream_tx_by_tx_hash(
    db: &DatabaseConnection,
    tx_hash: &str,
) -> Result<Option<stream_tx::Model>, DbErr> {
    stream_tx::Entity::find()
        .filter(stream_tx::Column::TxHash.eq(tx_hash))
        .one(db)
        .await
}

pub async fn create_stream_tx(
    db: &DatabaseConnection,
    params: CreateStreamTxParams,
) -> Result<(), DbErr> {
    let stream_tx = stream_tx::ActiveModel {
        tx_hash: Set(params.tx_hash),
        action: Set(params.action),
        context: Set(params.context),
        event: Set(params.event),
        date: Set(params.date),
        is_failure: Set(params.is_failure),
        message: Set(params.message),
        ..Default::default()
    };

    stream_tx::Entity::insert(stream_tx).exec(db).await?;

    Ok(())
}

pub struct CreateStreamTxParams {
    pub tx_hash: String,
    pub action: String,
    pub event: serde_json::Value,
    pub context: StreamContext,
    pub date: DateTimeWithTimeZone,
    pub is_failure: bool,
    pub message: Option<String>,
}

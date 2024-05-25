use crate::{
    database::{
        repository::{
            collection::{self as CollectionRespository, CreateCollectionParams},
            nft::{self as NftRepository, CreateNftParams},
            nft_activity::{self as NftActivityRepository, CreateNftActivityParams},
            transaction::{self as TransactionRepository, CreateTransactionParams},
            user_point::{self as UserPointRepository, CreateUserPointParams},
        },
        LoyaltyPointKind, Marketplace, NftActivityKind,
    },
    error::AppError,
    service::{get_collection_metadata, get_nft_metadata, CosmosClient},
};
use sea_orm::{
    prelude::{DateTimeUtc, Decimal},
    DatabaseConnection, DatabaseTransaction,
};
use std::str::FromStr;

pub async fn create_collection_if_not_exist(
    db: &DatabaseConnection,
    client: &CosmosClient,
    address: String,
    royalty: Option<f32>,
) -> Result<(), AppError> {
    let collection = CollectionRespository::find_by_address(db, &address).await?;

    if collection.is_some() {
        return Ok(());
    }

    let metadata = get_collection_metadata(&address).await?;
    let supply = client.get_cw721_contract_supply(&address).await?;
    let info = client.get_cw721_contract_info(&address).await?;

    CollectionRespository::create(
        db,
        CreateCollectionParams {
            address,
            symbol: info.symbol,
            name: info.name,
            metadata,
            supply: supply.count as i32,
            royalty: royalty
                .map(Decimal::from_f32_retain)
                .map(Option::unwrap_or_default),
        },
    )
    .await?;

    Ok(())
}

// only update owner from cw721 stream
pub async fn create_nft_or_update_owner_or_just_find(
    db: &DatabaseConnection,
    client: &CosmosClient,
    token_address: String,
    token_id: String,
    owner: Option<String>,
) -> Result<i32, AppError> {
    let nft = NftRepository::find_by_address_and_token_id(db, &token_address, &token_id).await?;

    if let Some(nft) = nft {
        NftRepository::update_owner(db, &token_address, &token_id, owner).await?;

        return Ok(nft.id);
    }

    let info = client.get_nft_info(&token_address, &token_id).await?;

    let metadata = get_nft_metadata(&info.token_uri).await?;

    create_collection_if_not_exist(
        db,
        client,
        token_address.to_owned(),
        info.extension
            .map(|ex| ex.royalty_percentage.unwrap_or_default()),
    )
    .await?;

    let nft_id = NftRepository::create(
        db,
        CreateNftParams {
            token_address,
            token_id,
            token_uri: info.token_uri,
            description: metadata.description,
            image: metadata.image,
            name: metadata.name,
            owner_address: owner,
            traits: metadata.attributes,
        },
    )
    .await?;

    Ok(nft_id)
}

pub async fn create_activity_transaction_and_point_on_sale(
    db: &DatabaseTransaction,
    params: CreateActivityTransactionAndPointOnSaleParams,
) -> Result<&DatabaseTransaction, AppError> {
    let price =
        Decimal::from_str(&params.price).map_err(|e| AppError::Unexpected(e.to_string()))?;

    let point = i32::from_str(&params.price)
        .map(|p| p / 1_000_000)
        .map_err(|e| AppError::Unexpected(e.to_string()))?;

    NftActivityRepository::create(
        &db,
        CreateNftActivityParams {
            buyer_address: Some(params.buyer.to_owned()),
            seller_address: Some(params.seller.to_owned()),
            created_date: params.date,
            denom: params.denom,
            event_kind: NftActivityKind::Sale,
            marketplace: params.marketplace.to_owned(),
            metadata: params.metadata,
            nft_id: params.nft_id,
            price,
            tx_hash: params.tx_hash.to_owned(),
        },
    )
    .await?;

    TransactionRepository::create(
        &db,
        CreateTransactionParams {
            buyer_address: params.buyer.to_owned(),
            seller_address: params.seller.to_owned(),
            collection_address: params.collection_address,
            created_date: params.date,
            marketplace: params.marketplace,
            tx_hash: params.tx_hash,
            volume: price,
        },
    )
    .await?;

    UserPointRepository::create(
        &db,
        CreateUserPointParams {
            date: params.date,
            kind: LoyaltyPointKind::Buy,
            point,
            wallet_address: params.buyer,
        },
    )
    .await?;

    UserPointRepository::create(
        &db,
        CreateUserPointParams {
            date: params.date,
            kind: LoyaltyPointKind::Sell,
            point,
            wallet_address: params.seller,
        },
    )
    .await?;

    Ok(db)
}

pub struct CreateActivityTransactionAndPointOnSaleParams {
    pub buyer: String,
    pub date: DateTimeUtc,
    pub denom: String,
    pub nft_id: i32,
    pub price: String,
    pub seller: String,
    pub tx_hash: String,
    pub collection_address: String,
    pub metadata: serde_json::Value,
    pub marketplace: Marketplace,
}

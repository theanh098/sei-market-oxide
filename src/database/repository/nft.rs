use crate::{
    database::entity::{
        listing_nft, nft, nft_trait,
        prelude::{ListingNft, Nft, NftTrait},
        sea_orm_active_enums::{Marketplace, SaleType},
    },
    service::NftAttribute,
};
use sea_orm::{
    prelude::{DateTimeUtc, Decimal},
    sea_query::OnConflict,
    ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

pub async fn find_by_address_and_token_id(
    db: &DatabaseConnection,
    token_address: &str,
    token_id: &str,
) -> Result<Option<nft::Model>, DbErr> {
    Nft::find()
        .filter(nft::Column::TokenAddress.eq(token_address))
        .filter(nft::Column::TokenId.eq(token_id))
        .one(db)
        .await
}

pub async fn find_listing_by_nft_id(
    db: &DatabaseConnection,
    nft_id: i32,
) -> Result<Option<listing_nft::Model>, DbErr> {
    ListingNft::find()
        .filter(listing_nft::Column::NftId.eq(nft_id))
        .one(db)
        .await
}

pub async fn update_owner(
    db: &DatabaseConnection,
    token_address: &str,
    token_id: &str,
    owner: Option<String>,
) -> Result<(), DbErr> {
    let nft = nft::ActiveModel {
        owner_address: Set(owner),
        ..Default::default()
    };

    Nft::update_many()
        .set(nft)
        .filter(nft::Column::TokenAddress.eq(token_address))
        .filter(nft::Column::TokenId.eq(token_id))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn create(db: &DatabaseConnection, params: CreateNftParams) -> Result<i32, DbErr> {
    let txn = db.begin().await?;

    let nft = nft::ActiveModel {
        token_address: Set(params.token_address),
        token_id: Set(params.token_id),
        token_uri: Set(params.token_uri),
        description: Set(params.description),
        name: Set(params.name),
        owner_address: Set(params.owner_address),
        image: Set(params.image),
        ..Default::default()
    };

    let nft_id = Nft::insert(nft)
        .on_conflict(
            OnConflict::columns([nft::Column::TokenAddress, nft::Column::TokenId])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?
        .last_insert_id;

    let traits = params.traits.unwrap_or_default().into_iter().map(
        |NftAttribute {
             trait_type,
             r#type,
             value,
             display_type,
         }| nft_trait::ActiveModel {
            nft_id: Set(nft_id),
            attribute: Set(trait_type.unwrap_or(r#type.unwrap_or("unknown".to_string()))),
            display_type: Set(display_type.map(|v| v.to_string())),
            value: Set(value
                .map(|v| v.to_string())
                .unwrap_or("unknown".to_string())),
            ..Default::default()
        },
    );

    NftTrait::insert_many(traits)
        .on_empty_do_nothing()
        .exec(db)
        .await?;

    txn.commit().await?;

    Ok(nft_id)
}

pub async fn create_pallet_listing(
    tx: &DatabaseTransaction,
    params: CreatePalletListingParams,
) -> Result<(), DbErr> {
    let CreatePalletListingParams {
        amount,
        denom,
        nft_id,
        tx_hash,
        created_date,
        collection_address,
        expiration_time,
        seller,
    } = params;

    let listing = listing_nft::ActiveModel {
        collection_address: Set(collection_address),
        created_date: Set(created_date.into()),
        denom: Set(denom),
        expiration_time: Set(expiration_time),
        market: Set(Marketplace::Pallet),
        nft_id: Set(nft_id),
        sale_type: Set(SaleType::Fixed),
        seller_address: Set(seller),
        price: Set(amount),
        tx_hash: Set(tx_hash),
        ..Default::default()
    };

    ListingNft::insert(listing)
        .on_conflict(
            OnConflict::column(listing_nft::Column::NftId)
                .do_nothing()
                .to_owned(),
        )
        .exec(tx)
        .await?;

    Ok(())
}

pub async fn delete_listing_if_exist(tx: &DatabaseTransaction, nft_id: i32) -> Result<(), DbErr> {
    ListingNft::delete_many()
        .filter(listing_nft::Column::NftId.eq(nft_id))
        .exec(tx)
        .await?;

    Ok(())
}

pub struct CreateNftParams {
    pub token_address: String,
    pub token_id: String,
    pub token_uri: String,
    pub name: Option<String>,
    pub image: Option<String>,
    pub traits: Option<Vec<NftAttribute>>,
    pub description: Option<String>,
    pub owner_address: Option<String>,
}

pub struct CreatePalletListingParams {
    pub nft_id: i32,
    pub collection_address: String,
    pub tx_hash: String,
    pub denom: String,
    pub amount: Decimal,
    pub created_date: DateTimeUtc,
    pub seller: String,
    pub expiration_time: Option<i32>,
}

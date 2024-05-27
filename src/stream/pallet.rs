use super::shared::{
    create_activity_transaction_and_point_on_sale, create_nft_or_update_owner_or_just_find,
    find_attribute, to_utf8, CreateActivityTransactionAndPointOnSaleParams, Event, Transaction,
};
use crate::{
    database::{
        repository::{
            nft::{self as NftRepository, CreatePalletListingParams},
            nft_activity::{self as NftActivityRepository, CreateNftActivityParams},
            tracing::{self as TracingRepository, CreateStreamTxParams},
        },
        Marketplace, NftActivityKind, StreamContext,
    },
    error::AppError,
    service::{CosmosClient, PalletListing},
};
use chrono::{DateTime, Utc};
use sea_orm::{prelude::Decimal, DatabaseConnection, TransactionTrait};
use std::str::FromStr;
use tendermint_rpc::endpoint::tx;

static CREATE_AUCTION_ACTION: &'static str = "wasm-create_auction";
static BUY_NOW_AUCTION: &'static str = "wasm-buy_now";
static CANCEL_AUCTION: &'static str = "wasm-cancel_auction";

pub async fn tx_handler(db: &DatabaseConnection, client: &CosmosClient, tx: Transaction) {
    let Transaction { tx_hash, events } = tx;

    let events = retrieve_pallet_events(events);

    for event in events {
        let action = &event.r#type;

        let result = if action == CREATE_AUCTION_ACTION {
            handle_create_auction(db, client, &event, &tx_hash).await
        } else if action == BUY_NOW_AUCTION {
            handle_buy_now(db, client, &event, &tx_hash).await
        } else if action == CANCEL_AUCTION {
            handle_cancel_auction(db, client, &event, &tx_hash).await
        } else {
            println!("unexpected action {} event {:#?}", action, event);
            Ok(())
        };

        if let Err(error) = result {
            TracingRepository::create_stream_tx(
                db,
                CreateStreamTxParams {
                    action: action.to_owned(),
                    context: StreamContext::Pallet,
                    date: Utc::now().into(),
                    event: serde_json::json!(event),
                    is_failure: true,
                    tx_hash: tx_hash.to_owned(),
                    message: Some(error.to_string()),
                },
            )
            .await
            .unwrap_or_else(|e| eprintln!("unexpected error when create tracing tx {}", e));

            eprintln!(
                "unexpected error when handle pallet event {} {} \n>>{}",
                action, tx_hash, error
            );
        } else {
            TracingRepository::create_stream_tx(
                db,
                CreateStreamTxParams {
                    action: action.to_owned(),
                    context: StreamContext::Pallet,
                    date: Utc::now().into(),
                    event: serde_json::json!(event),
                    is_failure: false,
                    tx_hash: tx_hash.to_owned(),
                    message: None,
                },
            )
            .await
            .unwrap_or_else(|e| eprintln!("unexpected error when create tracing tx {}", e));

            println!("done handle pallet event {} {}", action, tx_hash);
        }
    }
}

async fn handle_create_auction(
    db: &DatabaseConnection,
    client: &CosmosClient,
    event: &Event,
    tx_hash: &String,
) -> Result<(), AppError> {
    let token_address = find_attribute(event, "collection_address")?;
    let token_id = find_attribute(event, "token_id")?;

    let nft_id = create_nft_or_update_owner_or_just_find(
        db,
        client,
        token_address.to_owned(),
        token_id.to_owned(),
        None,
    )
    .await?;

    let pallet_listing = client.get_pallet_listing(&token_address, &token_id).await?;

    let PalletListing { auction, owner } = pallet_listing;

    let Some(auction) = auction else {
        return Ok(());
    };

    let price = auction.prices.get(0).ok_or(AppError::Unexpected(
        "can not parse pallet listing price".to_owned(),
    ))?;

    let amount =
        Decimal::from_str(&price.amount).map_err(|e| AppError::Unexpected(e.to_string()))?;

    let created_date = DateTime::from_timestamp(auction.created_at as i64, 0).ok_or(
        AppError::Unexpected("can not parse pallet listing created_date".to_owned()),
    )?;

    let tx = db.begin().await?;

    NftRepository::create_pallet_listing(
        &tx,
        CreatePalletListingParams {
            amount,
            created_date: created_date.into(),
            denom: "usei".to_string(),
            nft_id,
            tx_hash: tx_hash.to_owned(),
            collection_address: token_address,
            expiration_time: Some(auction.expiration_time as i32),
            seller: owner.to_owned(),
        },
    )
    .await?;

    NftActivityRepository::create(
        &tx,
        CreateNftActivityParams {
            nft_id,
            created_date: created_date.into(),
            denom: "usei".to_string(),
            event_kind: NftActivityKind::List,
            marketplace: Marketplace::Pallet,
            metadata: serde_json::json!({}),
            price: amount,
            seller_address: Some(owner),
            tx_hash: tx_hash.to_owned(),
            buyer_address: None,
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

async fn handle_buy_now(
    db: &DatabaseConnection,
    client: &CosmosClient,
    event: &Event,
    tx_hash: &String,
) -> Result<(), AppError> {
    let token_address = find_attribute(event, "collection_address")?;
    let token_id = find_attribute(event, "token_id")?;

    let nft_id = create_nft_or_update_owner_or_just_find(
        db,
        client,
        token_address.to_owned(),
        token_id,
        None,
    )
    .await?;

    let db_listing = NftRepository::find_listing_by_nft_id(db, nft_id).await?;

    let Some(db_listing) = db_listing else {
        return Ok(());
    };

    let tx = client.get_tx(&tx_hash).await?;

    let buyer = find_buyer_address_from_tx(&tx).ok_or(AppError::Unexpected(format!(
        "can not get buyer from tx {} in buy now event",
        tx_hash,
    )))?;

    let db = db.begin().await?;

    NftRepository::delete_listing_if_exist(&db, nft_id).await?;

    create_activity_transaction_and_point_on_sale(
        &db,
        CreateActivityTransactionAndPointOnSaleParams {
            buyer,
            collection_address: token_address,
            date: Utc::now().into(),
            denom: "usei".to_string(),
            marketplace: Marketplace::Pallet,
            metadata: serde_json::json!({}),
            nft_id,
            price: db_listing.price.to_string(),
            seller: db_listing.seller_address,
            tx_hash: tx_hash.to_owned(),
        },
    )
    .await?;

    Ok(())
}

async fn handle_cancel_auction(
    db: &DatabaseConnection,
    client: &CosmosClient,
    event: &Event,
    tx_hash: &String,
) -> Result<(), AppError> {
    let token_address = find_attribute(event, "collection_address")?;
    let token_id = find_attribute(event, "token_id")?;

    let nft_id =
        create_nft_or_update_owner_or_just_find(db, client, token_address, token_id, None).await?;

    let db_listing = NftRepository::find_listing_by_nft_id(db, nft_id).await?;

    let Some(db_listing) = db_listing else {
        return Ok(());
    };

    let tx = db.begin().await?;

    NftRepository::delete_listing_if_exist(&tx, nft_id).await?;

    NftActivityRepository::create(
        &tx,
        CreateNftActivityParams {
            buyer_address: None,
            created_date: Utc::now().into(),
            denom: "usei".to_string(),
            event_kind: NftActivityKind::Delist,
            marketplace: Marketplace::Pallet,
            metadata: serde_json::json!({}),
            nft_id,
            price: db_listing.price,
            seller_address: Some(db_listing.seller_address),
            tx_hash: tx_hash.to_owned(),
        },
    )
    .await?;

    tx.commit().await?;

    Ok(())
}

fn find_buyer_address_from_tx(tx: &tx::Response) -> Option<String> {
    tx.tx_result
        .events
        .iter()
        .find_map(|e| {
            if e.kind != "wasm" {
                None
            } else {
                e.attributes.iter().find(|attribute| {
                    attribute
                        .key_str()
                        .is_ok_and(|key| to_utf8(key) == "recipent")
                })
            }
        })
        .and_then(|attribute| attribute.value_str().ok())
        .map(str::to_owned)
}

fn retrieve_pallet_events(events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .filter(|Event { r#type, .. }| {
            r#type == BUY_NOW_AUCTION || r#type == CANCEL_AUCTION || r#type == CREATE_AUCTION_ACTION
        })
        .collect()
}

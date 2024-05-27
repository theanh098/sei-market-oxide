pub mod cw721;
pub mod mrkt;
pub mod pallet;
mod shared;

use self::shared::Transaction;
use crate::{error::AppError, r#static::PALLET_CONTRACT_ADDRESS, service::CosmosClient};
use futures_util::{SinkExt, StreamExt};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde_json::Value;
use std::future::Future;
use tendermint_rpc::query::{EventType, Query};
use tokio_tungstenite::{connect_async, tungstenite::Message};

static INGORE_MESSAGE: &'static str = "{\"jsonrpc\":\"2.0\",\"id\":\"0\",\"result\":{}}";

pub async fn cw721_stream() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let rpc_url = std::env::var("RPC_URL").expect("rpc_url must be set");
    let cosmos_client =
        CosmosClient::from(tendermint_rpc::HttpClient::new(rpc_url.as_str()).unwrap());

    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await.unwrap();

    let query = Query::from(EventType::Tx)
        .and_exists("wasm.action")
        .and_exists("wasm._contract_address")
        .and_exists("wasm.token_id");

    let msg = create_subcribe_message(query);

    loop {
        if let Err(error) = stream_handler(&db, &cosmos_client, &msg, cw721::tx_handler).await {
            eprintln!("{}", error)
        }
    }
}

pub async fn pallet_stream() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let rpc_url = std::env::var("RPC_URL").expect("rpc_url must be set");
    let cosmos_client =
        CosmosClient::from(tendermint_rpc::HttpClient::new(rpc_url.as_str()).unwrap());

    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await.unwrap();

    let query =
        Query::from(EventType::Tx).and_eq("execute._contract_address", PALLET_CONTRACT_ADDRESS);

    let msg = create_subcribe_message(query);

    loop {
        if let Err(error) = stream_handler(&db, &cosmos_client, &msg, pallet::tx_handler).await {
            eprintln!("{}", error)
        }
    }
}

async fn stream_handler<'r, F, Fut>(
    db: &'r DatabaseConnection,
    cosmos_client: &'r CosmosClient,
    msg_subcribe: &Message,
    tx_handler: F,
) -> Result<(), AppError>
where
    F: Fn(&'r DatabaseConnection, &'r CosmosClient, Transaction) -> Fut,
    Fut: Future<Output = ()> + 'r,
{
    let wss_url = std::env::var("WSS_URL").expect("wss_url must be set");

    let (ws_stream, _) = connect_async(wss_url).await?;

    let (mut write, mut read) = ws_stream.split();

    write.send(msg_subcribe.to_owned()).await?;

    while let Some(message) = read.next().await {
        if let Message::Text(message) = message? {
            if message != INGORE_MESSAGE {
                let message = serde_json::from_str::<Value>(&message)?;
                let tx_result = Transaction::try_from_value(message)?;
                tx_handler(db, cosmos_client, tx_result).await
            } else {
                // we skip first message, so this time is perfect to tell that stream is working
                println!("listening stream")
            }
        }
    }

    Ok(())
}

fn create_subcribe_message(query: Query) -> Message {
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "subscribe",
        "id": "0",
        "params": {
          "query": query.to_string()
        }
    });

    let msg = Message::text(msg.to_string());

    msg
}

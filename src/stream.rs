pub mod cw721;
pub mod mrkt;
pub mod pallet;
mod shared;

use crate::{error::AppError, service::CosmosClient};
use base64::{prelude::BASE64_STANDARD, Engine};
use futures_util::{SinkExt, StreamExt};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::future::Future;
use tendermint_rpc::query::Query;
use tokio_tungstenite::{connect_async, tungstenite::Message};

static INGORE_MESSAGE: &'static str = "{\"jsonrpc\":\"2.0\",\"id\":\"0\",\"result\":{}}";

pub trait FromJsonValue
where
    Self: Sized,
{
    fn try_from_value(value: serde_json::Value) -> Result<Self, AppError>;
}

#[derive(Debug)]
pub struct Transaction {
    pub tx_hash: String,
    pub events: Vec<Event>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Event {
    pub r#type: String,
    pub attributes: Vec<Attribute>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

pub async fn stream_handler<'r, F, Fut>(
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

pub fn find_attribute(event: &Event, key: &str) -> Result<String, AppError> {
    event
        .attributes
        .iter()
        .find(|Attribute { key: k, .. }| k == key)
        .map(|attribute| attribute.value.to_owned())
        .ok_or(AppError::Unexpected(format!("missing attribute {}", key)))
}

pub fn create_subcribe_message(query: Query) -> Message {
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

fn to_utf8(base64: &str) -> String {
    let buffer = BASE64_STANDARD.decode(base64).unwrap_or_default();
    String::from_utf8(buffer).unwrap_or_default()
}

impl FromJsonValue for Transaction {
    fn try_from_value(value: serde_json::Value) -> Result<Transaction, AppError> {
        let tx_hash = value
            .get("result")
            .and_then(|v| v.get("events"))
            .and_then(|v| v.get("tx.hash"))
            .and_then(|v| v.get(0))
            .ok_or(AppError::Unexpected("missing tx.hash attribute".to_owned()))?;

        let Value::String(tx_hash) = tx_hash else {
            return Err(AppError::Unexpected(
                "missing result.events[tx.hash] is not string".to_owned(),
            ));
        };

        let events = value
            .get("result")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.get("TxResult"))
            .and_then(|v| v.get("result"))
            .and_then(|v| v.get("events"))
            .ok_or(AppError::Unexpected(
                "missing result.data.value.TxResult.result.events attribute".to_owned(),
            ))?;

        let events = serde_json::from_value::<Vec<Event>>(events.to_owned())?
            .into_iter()
            .map(|Event { attributes, r#type }| Event {
                r#type,
                attributes: attributes
                    .into_iter()
                    .map(|Attribute { key, value }| Attribute {
                        key: to_utf8(&key),
                        value: to_utf8(&value),
                    })
                    .collect(),
            })
            .collect();

        Ok(Transaction {
            tx_hash: tx_hash.to_owned(),
            events,
        })
    }
}

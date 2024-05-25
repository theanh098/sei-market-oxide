#![allow(unused_imports)]
#![allow(dead_code)]

mod database;
mod error;
mod openapi;
mod server;
mod service;
mod stream;

use crate::server::api;
use axum::{routing::get, Json, Router};

use sea_orm::{ConnectOptions, Database};
use server::extract::state::AppState;
use service::CosmosClient;
use stream::{create_subcribe_message, cw721, pallet, stream_handler};
use tendermint_rpc::query::{EventType, Query};
use tokio::net::TcpListener;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

static PALLET_API_URL: &'static str = "https://api.pallet.exchange/api";

static PALLET_CONTRACT_ADDRESS: &'static str =
    "sei152u2u0lqc27428cuf8dx48k8saua74m6nql5kgvsu4rfeqm547rsnhy4y9";

static MRKT_CONTRACT_ADDRESS: &'static str =
    "sei1dkp90y3jpp2dres2ssp5rak2k6mc7l4nsxz58nktxjsxqp88fcasmrr672";

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

pub async fn server() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let redis_url = "redis://127.0.0.1/";

    let address = "0.0.0.0:8098";

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .route("/api/v1/", get(|| async { "Hello, 🦀!" }))
        .route("/api/v1/collections", get(api::collection::get_collections))
        .with_state(AppState::init(&db_url, redis_url).await);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    println!("🦀 server is running on port {}", address);

    axum::serve(listener, app).await.unwrap();
}

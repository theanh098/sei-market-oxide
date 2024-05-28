mod listener;
mod trigger;

use sea_orm::Database;
use serde_json::Value;
use trigger::{create_stream_tx_trigger, Channel};

pub async fn watcher() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let db = Database::connect(&db_url).await.unwrap();

    create_stream_tx_trigger(&db)
        .await
        .unwrap_or_else(|e| eprintln!("fail when create stream tx trigger >>{}", e));

    listener::Listener::new()
        .add_watcher(Channel::StreamTx, &|_db, payload: Value| {
            Box::pin(async move {
                println!("payload: {:#?}", payload);
                Ok(())
            })
        })
        .start(&db)
        .await
        .unwrap_or_else(|e| eprintln!("{}", e));
}

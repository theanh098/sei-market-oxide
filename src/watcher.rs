mod listener;

use sea_orm::Database;
use serde_json::Value;

pub async fn watcher() {
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let db = Database::connect(&db_url).await.unwrap();

    listener::Listener::new()
        .add_watcher("pp", &|_db, _payload: Value| {
            Box::pin(async move { Ok(()) })
        })
        .add_watcher("pp", &|_db, _payload: Value| {
            Box::pin(async move { Ok(()) })
        })
        .start(&db)
        .await;
}

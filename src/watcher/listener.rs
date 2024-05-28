use sea_orm::DatabaseConnection;
use serde::de::DeserializeOwned;
use sqlx::postgres::PgListener;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::error::AppError;

type Worker<'r, P> =
    dyn Fn(&'r DatabaseConnection, P) -> Pin<Box<dyn Future<Output = Result<(), AppError>>>>;

pub struct Listener<'r, P> {
    channels: HashMap<&'static str, &'r Worker<'r, P>>,
}

impl<'r, P> Listener<'r, P>
where
    P: DeserializeOwned + Sized + Debug,
{
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn add_watcher(mut self, name: &'static str, worker: &'r Worker<'r, P>) -> Self {
        self.channels.insert(name, worker);
        self
    }

    pub async fn start(&self, db: &'r DatabaseConnection) {
        let pool = db.get_postgres_connection_pool();
        let mut listener = PgListener::connect_with(pool).await.unwrap();
        let channels: Vec<&str> = self.channels.keys().into_iter().map(|key| *key).collect();

        listener.listen_all(channels).await.unwrap();

        loop {
            while let Some(notification) = listener.try_recv().await.unwrap() {
                let chanel = notification.channel();
                let worker = self.channels.get(chanel).unwrap();

                let payload_string = notification.payload().to_owned();
                let payload = serde_json::from_str::<P>(&payload_string).unwrap();

                worker(db, payload).await.unwrap();
            }
        }
    }
}

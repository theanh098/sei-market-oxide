use sea_orm::DatabaseConnection;
use serde::de::DeserializeOwned;
use sqlx::postgres::PgListener;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::error::AppError;

use super::trigger::Channel;

type Worker<'r, P> =
    dyn Fn(&'r DatabaseConnection, P) -> Pin<Box<dyn Future<Output = Result<(), AppError>>>>;

pub struct Listener<'r, P> {
    channels: HashMap<Channel, &'r Worker<'r, P>>,
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

    pub fn add_watcher(mut self, channel: Channel, worker: &'r Worker<'r, P>) -> Self {
        self.channels.insert(channel, worker);
        self
    }

    pub async fn start(&self, db: &'r DatabaseConnection) -> Result<(), AppError> {
        let pool = db.get_postgres_connection_pool();
        let mut listener = PgListener::connect_with(pool).await?;
        let channels: Vec<&str> = self
            .channels
            .keys()
            .into_iter()
            .map(|key| key.to_str())
            .collect();

        listener.listen_all(channels.clone()).await?;

        println!("🦀 watching on the channels {:?}", channels);

        loop {
            while let Some(notification) = listener.try_recv().await? {
                let chanel = notification.channel();
                let worker = self.channels.get(&Channel::from_str(&chanel)).unwrap();

                let payload_string = notification.payload();
                let payload = serde_json::from_str::<P>(&payload_string)?;

                worker(db, payload).await?;
            }
        }
    }
}

use crate::error::AppError;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use deadpool_redis::{Config, Runtime};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub type RedisConnection = deadpool_redis::Connection;

pub struct Redis(pub RedisConnection);
pub struct Postgres(pub DatabaseConnection);

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis_pool: deadpool_redis::Pool,
}

#[async_trait]
impl<S> FromRequestParts<S> for Postgres
where
    S: Send + Sync,
    DatabaseConnection: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let connection = DatabaseConnection::from_ref(state);

        Ok(Self(connection))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Redis
where
    S: Send + Sync,
    deadpool_redis::Pool: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let connection = deadpool_redis::Pool::from_ref(state).get().await?;

        Ok(Self(connection))
    }
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(app_state: &AppState) -> DatabaseConnection {
        app_state.db.clone()
    }
}

impl FromRef<AppState> for deadpool_redis::Pool {
    fn from_ref(app_state: &AppState) -> deadpool_redis::Pool {
        app_state.redis_pool.clone()
    }
}

impl AppState {
    pub async fn init(db_url: &str, redis_url: &str) -> Self {
        let mut opt = ConnectOptions::new(db_url);

        opt.sqlx_logging(false);

        let database_connection = Database::connect(opt).await.unwrap();

        let redis_pool = Config::from_url(redis_url)
            .create_pool(Some(Runtime::Tokio1))
            .unwrap();

        Self {
            db: database_connection,
            redis_pool,
        }
    }
}

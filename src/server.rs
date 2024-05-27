pub mod api;
pub mod deserialization;
mod extract;
mod openapi;
mod serialization;

use crate::server::extract::state::AppState;
use axum::{routing::get, Router};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub async fn server() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");
    let redis_url = "redis://127.0.0.1/";

    let address = "0.0.0.0:8098";

    let app = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi::ApiDoc::openapi()))
        .route("/api/v1/", get(|| async { "Hello, 🦀!" }))
        .route("/api/v1/collections", get(api::collection::get_collections))
        .route("/api/v1/leaderboard", get(api::leaderboard::get_leaderboad))
        .with_state(AppState::init(&db_url, redis_url).await);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    println!("🦀 server is running on port {}", address);

    axum::serve(listener, app).await.unwrap();
}

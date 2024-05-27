use axum::{
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serde_json::json;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    // external
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumFormRejection(#[from] axum::extract::rejection::FormRejection),

    #[error(transparent)]
    AxumQueryRejection(#[from] axum::extract::rejection::QueryRejection),

    #[error(transparent)]
    AxumPayloadRejection(#[from] axum::extract::rejection::JsonRejection),

    #[error("{0}")]
    BadRequestError(String),

    #[error("{0}")]
    Unauthorized(String),

    // internal
    #[error("{0}")]
    Unexpected(String),

    #[error(transparent)]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Database(#[from] sea_orm::error::DbErr),

    #[error(transparent)]
    Redis(#[from] deadpool_redis::PoolError),

    #[error(transparent)]
    HttpRequest(#[from] reqwest::Error),

    #[error(transparent)]
    Cosmos(#[from] crate::service::CosmosClientError),

    #[error("{0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Validation(error) => (
                StatusCode::BAD_REQUEST,
                to_json(StatusCode::BAD_REQUEST, error.to_string()),
            ),
            AppError::AxumFormRejection(rejection) => (
                StatusCode::BAD_REQUEST,
                to_json(StatusCode::BAD_REQUEST, rejection.to_string()),
            ),
            AppError::AxumQueryRejection(rejection) => (
                StatusCode::BAD_REQUEST,
                to_json(StatusCode::BAD_REQUEST, rejection.to_string()),
            ),
            AppError::AxumPayloadRejection(rejection) => (
                StatusCode::BAD_REQUEST,
                to_json(StatusCode::BAD_REQUEST, rejection.to_string()),
            ),
            AppError::BadRequestError(reason) => (
                StatusCode::BAD_REQUEST,
                to_json(StatusCode::BAD_REQUEST, reason),
            ),
            AppError::Unauthorized(reason) => (
                StatusCode::UNAUTHORIZED,
                to_json(StatusCode::UNAUTHORIZED, reason),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                to_json(StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ),
        }
        .into_response()
    }
}

fn to_json(code: StatusCode, message: String) -> Json<serde_json::Value> {
    Json(json!({
        "code": code.as_u16(),
        "message": message,
        "status": code.to_string()
    }))
}

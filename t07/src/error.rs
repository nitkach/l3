use axum::{http::StatusCode, response::IntoResponse};
use log::error;

#[derive(thiserror::Error, Debug)]
pub(crate) enum AppError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error("{0} not found")]
    NotFound(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Redis(redis) => {
                error!("{redis}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_owned(),
                )
            }
            AppError::Other(other) => {
                error!("{other:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "something went wrong".to_owned(),
                )
            }
        }
        .into_response()
    }
}

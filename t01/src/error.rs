use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    #[error("{0}")]
    Authenthication(String),

    #[error("{0}")]
    NotFound(String),

    #[error(transparent)]
    JwtToken(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::Authenthication(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::JwtToken(_) | AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "something went wrong".to_owned(),
            ),
        }
        .into_response()
    }
}

use axum::{http::StatusCode, response::IntoResponse};

pub(crate) struct AppError {
    code: StatusCode,
    source: anyhow::Error,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        todo!()
    }
}

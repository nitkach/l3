use axum::{response::IntoResponse, Json};
use http::StatusCode;
use log::error;

pub(crate) struct AppError {
    code: StatusCode,
    kind: ErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("{0}")]
    Registration(String),

    #[error("{0}")]
    Authenthication(String),

    #[error("{0}")]
    Forbidden(String),

    #[error(transparent)]
    JwtToken(#[from] jsonwebtoken::errors::Error),

    #[error("The requested page does not exist.")]
    PageNotFound,

    #[error("The requested post does not exist.")]
    PostNotFound,

    #[error("User does not exist.")]
    UserNotFound,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    fn new(code: StatusCode, kind: ErrorKind) -> Self {
        Self { code, kind }
    }

    pub(crate) fn user_already_exist() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            ErrorKind::Registration("User already exists".to_owned()),
        )
    }

    pub(crate) fn authenthication(message: &str) -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            ErrorKind::Authenthication(message.to_owned()),
        )
    }

    pub(crate) fn jwt_token(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                Self::new(StatusCode::UNAUTHORIZED, ErrorKind::JwtToken(error))
            }
            _ => Self::new(StatusCode::BAD_REQUEST, ErrorKind::JwtToken(error)),
        }
    }

    pub(crate) fn other(error: anyhow::Error) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            kind: ErrorKind::Other(error),
        }
    }

    pub(crate) fn forbidden(message: &str) -> Self {
        Self {
            code: StatusCode::FORBIDDEN,
            kind: ErrorKind::Forbidden(message.to_owned()),
        }
    }

    pub(crate) fn page_not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::PageNotFound)
    }

    pub(crate) fn post_not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::PostNotFound)
    }

    pub(crate) fn user_not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, ErrorKind::UserNotFound)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let message = match self.kind {
            ErrorKind::JwtToken(jwt_error)
                if matches!(
                    jwt_error.kind(),
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature
                ) =>
            {
                "Session expired. Please, authorize again.".to_owned()
            }
            ErrorKind::Other(err) => {
                error!("{err:?}");
                "Something went wrong.".to_owned()
            }
            _ => self.kind.to_string(),
        };
        let json = serde_json::json!({
            "result": "err",
            "message": message
        });

        (code, Json(json)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            kind: ErrorKind::Other(err.into()),
        }
    }
}

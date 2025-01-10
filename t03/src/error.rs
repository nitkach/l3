use axum::response::IntoResponse;
use http::StatusCode;

#[derive(Debug)]
pub(crate) struct AppError {
    code: StatusCode,
    kind: ErrorKind,
}

impl AppError {
    pub(crate) fn new(code: StatusCode, kind: ErrorKind) -> Self {
        Self { code, kind }
    }

    pub(crate) fn with_kind(kind: ErrorKind) -> Self {
        match kind {
            client @ (ErrorKind::Room(_) | ErrorKind::User(_)) => Self {
                code: StatusCode::BAD_REQUEST,
                kind: client,
            },
            other @ ErrorKind::Other(_) => Self {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                kind: other,
            },
        }
    }

    pub(crate) fn bad_request(kind: ErrorKind) -> Self {
        Self {
            code: StatusCode::BAD_REQUEST,
            kind,
        }
    }

    pub(crate) fn not_found(kind: ErrorKind) -> Self {
        Self {
            code: StatusCode::NOT_FOUND,
            kind,
        }
    }

    pub(crate) fn conflict(kind: ErrorKind) -> Self {
        Self {
            code: StatusCode::CONFLICT,
            kind,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("{0}")]
    Room(String),

    #[error("{0}")]
    User(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let body = match self.kind {
            client @ (ErrorKind::Room(_) | ErrorKind::User(_)) => client.to_string(),
            ErrorKind::Other(_) => "something went wrong".to_owned(),
        };

        (code, body).into_response()
    }
}

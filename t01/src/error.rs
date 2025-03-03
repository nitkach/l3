use askama::Template;
use axum::response::IntoResponse;
use http::StatusCode;
use log::error;

pub(crate) struct AppError {
    code: StatusCode,
    kind: ErrorKind,
}

#[derive(Debug, Template)]
#[template(path = "error.askama.html")]
pub(crate) struct ErrorTemplate {
    code: StatusCode,
    message: String,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorKind {
    #[error("{0}")]
    Registration(String),

    #[error("Missing credentials")]
    Authenthication(String),

    #[error("User does not exists")]
    NotFound(String),

    #[error(transparent)]
    JwtToken(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub(crate) fn new(code: StatusCode, kind: ErrorKind) -> Self {
        Self { code, kind }
    }

    pub(crate) fn user_already_exist() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            ErrorKind::Registration("User already exists".to_owned()),
        )
    }

    pub(crate) fn authenthication(word: String) -> Self {
        todo!()
        // Self::new(StatusCode::NOT_FOUND, ErrorKind::NoDefinitionsFound(word))
    }

    pub(crate) fn not_found(message: &str) -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            ErrorKind::NotFound(message.to_owned()),
        )
    }

    pub(crate) fn jwt_token(error: jsonwebtoken::errors::Error) -> Self {
        todo!()
        // Self::new(
        //     StatusCode::NOT_FOUND,
        //     ErrorKind::NotFound(message.to_owned()),
        // )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let message = match self.kind {
            ErrorKind::Other(err) => {
                error!("{err:?}");
                "Something went wrong.".to_owned()
            }
            _ => self.kind.to_string(),
        };

        let template = ErrorTemplate { code, message };

        (code, askama_axum::into_response(&template)).into_response()
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

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Claims {
    pub(crate) sub: Ulid,
    pub(crate) exp: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RegisterRequest {
    pub(crate) login: String,
    pub(crate) password: String,
}

impl RegisterRequest {
    pub(crate) fn is_empty(&self) -> bool {
        self.login.is_empty() || self.password.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct LoginRequest {
    pub(crate) login: String,
    pub(crate) password: String,
}

impl LoginRequest {
    pub(crate) fn is_empty(&self) -> bool {
        self.login.is_empty() || self.password.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreatePostRequest {
    pub(crate) content: String,
}

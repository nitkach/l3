use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::utils::PasswordHash;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Claims {
    pub(crate) sub: i32,
    pub(crate) exp: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RegisterForm {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl RegisterForm {
    pub(crate) fn is_empty(&self) -> bool {
        self.username.is_empty() || self.password.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct LoginRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl LoginRequest {
    pub(crate) fn is_empty(&self) -> bool {
        self.username.is_empty() || self.password.is_empty()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreatePostRequest {
    pub(crate) content: String,
}

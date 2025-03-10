use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Claims {
    pub(crate) sub: i32,
    pub(crate) exp: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RegisterRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl RegisterRequest {
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
    pub(crate) title: String,
    pub(crate) content: String,
}

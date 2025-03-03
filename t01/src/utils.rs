use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PasswordHash(String);

impl PasswordHash {
    pub(crate) fn from_password(password: &str) -> bcrypt::BcryptResult<Self> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST).map(Self)
    }

    pub(crate) fn verify_password(&self, password: &str) -> Result<bool, bcrypt::BcryptError> {
        bcrypt::verify(password, &self.0)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<&tokio_postgres::Row> for PasswordHash {
    type Error = anyhow::Error;

    fn try_from(row: &tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self(row.try_get("password_hash")?))
    }
}

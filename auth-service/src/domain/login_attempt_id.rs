use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginAttemptId(String);

#[derive(thiserror::Error, Debug)]
#[error("Invalid Login Attempt Id: {0}")]
pub struct LoginAttemptIdError(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, LoginAttemptIdError> {
        Uuid::parse_str(&id)
            .map(|_| Self(id))
            .map_err(|e| LoginAttemptIdError(e.to_string()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(Uuid::now_v7().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

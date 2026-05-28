use crate::domain::Secret;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoginAttemptId(Secret);

#[derive(thiserror::Error, Debug)]
#[error("Invalid Login Attempt Id")]
pub struct LoginAttemptIdError;

impl LoginAttemptId {
    pub fn parse(id: &Secret) -> Result<Self, LoginAttemptIdError> {
        let raw_id = id.expose();
        uuid::Uuid::parse_str(raw_id)
            .map(|_| Self(id.to_owned()))
            .map_err(|_| LoginAttemptIdError)
    }

    #[must_use]
    pub fn as_secret(&self) -> &Secret {
        &self.0
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4().to_string().into())
    }
}

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.as_secret() == other.as_secret()
    }
}

impl Eq for LoginAttemptId {}

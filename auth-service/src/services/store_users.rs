use crate::domain::User;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum UserStoreError {
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("User was not found: {0}")]
    UserNotFound(String),
    #[error("User incorrect credentials: {0}")]
    IncorrectCredentials(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &str, raw_password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
        match user.password.verify_raw_password(raw_password).await {
            Ok(()) => Ok(()),
            Err(_) => Err(UserStoreError::IncorrectCredentials(email.to_string())),
        }
    }
}

#[derive(Clone)]
pub struct UserStoreType {
    inner: Arc<dyn UserStore>,
}

impl UserStoreType {
    pub fn new(inner: impl UserStore + 'static) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> Arc<dyn UserStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for UserStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserStoreType").finish_non_exhaustive()
    }
}

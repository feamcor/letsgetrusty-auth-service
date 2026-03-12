use crate::domain::Email;
use crate::domain::Secret;
use crate::domain::User;

#[derive(thiserror::Error, Debug)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User was not found")]
    UserNotFound,
    #[error("User incorrect credentials")]
    IncorrectCredentials,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type UserStoreResult<T> = Result<T, UserStoreError>;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&self, user: User) -> UserStoreResult<()>;
    async fn get_user(&self, email: &Email) -> UserStoreResult<User>;
    async fn validate_user(&self, email: &Email, password: &Secret) -> UserStoreResult<()> {
        let user = self.get_user(email).await?;
        match user.password.verify_password(password).await {
            Ok(()) => Ok(()),
            Err(_) => Err(UserStoreError::IncorrectCredentials),
        }
    }
}

#[derive(Clone)]
pub struct UserStoreType {
    inner: std::sync::Arc<dyn UserStore>,
}

impl UserStoreType {
    pub fn new(inner: impl UserStore + 'static) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> std::sync::Arc<dyn UserStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for UserStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserStoreType").finish_non_exhaustive()
    }
}

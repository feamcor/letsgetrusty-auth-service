use crate::domain::User;

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
pub trait UserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
}

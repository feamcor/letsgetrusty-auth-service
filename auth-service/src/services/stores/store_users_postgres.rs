use crate::domain::{Email, HashedPassword, User};
use crate::services::{UserStore, UserStoreError};
use sqlx::PgPool;
use tracing::error;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&self, user: User) -> Result<(), UserStoreError> {
        match sqlx::query!(
            r#"
            INSERT INTO auth.users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.as_ref(),
            user.password.expose(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(error)) => {
                let code = error.code();
                let code = code.as_deref().unwrap_or("unknown");
                if code == "23505" {
                    Err(UserStoreError::UserAlreadyExists(user.email.to_string()))
                } else {
                    error!("Unexpected error on adding user: {}: {}", code, error);
                    Err(UserStoreError::UnexpectedError(error.into()))
                }
            }
            Err(error) => {
                error!("Unexpected error on adding user: {}", error);
                Err(UserStoreError::UnexpectedError(error.into()))
            }
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match sqlx::query!(
            r#"
            SELECT users.email, users.password_hash, users.requires_2fa
            FROM auth.users
            WHERE users.email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(Some(record)) => Ok(User {
                email: Email::parse(&record.email)
                    .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
                password: HashedPassword::parse_password_hash(&record.password_hash)
                    .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
                requires_2fa: record.requires_2fa,
            }),
            Ok(None) => Err(UserStoreError::UserNotFound(email.to_string())),
            Err(error) => Err(UserStoreError::UnexpectedError(error.into())),
        }
    }
}

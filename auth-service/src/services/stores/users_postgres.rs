use crate::domain::Email;
use crate::domain::HashedPassword;
use crate::domain::User;
use crate::services::UserStore;
use crate::services::UserStoreError;
use crate::services::UserStoreResult;

pub struct PostgresUserStore {
    pool: sqlx::PgPool,
}

impl PostgresUserStore {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&self, user: User) -> UserStoreResult<()> {
        match sqlx::query!(
            r#"
            INSERT INTO auth.users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.as_secret().expose(),
            user.password.as_secret().expose(),
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
                    Err(UserStoreError::UserAlreadyExists)
                } else {
                    tracing::error!("Unexpected error on adding user: {}: {}", code, error);
                    Err(UserStoreError::UnexpectedError(error.into()))
                }
            }
            Err(error) => {
                tracing::error!("Unexpected error on adding user: {}", error);
                Err(UserStoreError::UnexpectedError(error.into()))
            }
        }
    }

    async fn get_user(&self, email: &Email) -> UserStoreResult<User> {
        match sqlx::query!(
            r#"
            SELECT users.email, users.password_hash, users.requires_2fa
            FROM auth.users
            WHERE users.email = $1
            "#,
            email.as_secret().expose()
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(Some(record)) => Ok(User {
                email: Email::parse(&record.email.into()).map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
                password: HashedPassword::parse_password_hash(&record.password_hash.into())
                    .map_err(|e| UserStoreError::UnexpectedError(e.into()))?,
                requires_2fa: record.requires_2fa,
            }),
            Ok(None) => Err(UserStoreError::UserNotFound),
            Err(error) => Err(UserStoreError::UnexpectedError(error.into())),
        }
    }
}

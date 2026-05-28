use crate::domain::Email;
use crate::domain::HashedPassword;
use crate::domain::Secret;
use crate::domain::User;
use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::PasswordHasher;
use argon2::Version;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use std::sync::LazyLock;

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

// Throwaway Argon2id hash used to equalize wall-clock time on the user-not-found branch of
// validate_user, eliminating the user-enumeration timing oracle. Generated once at first use
// with the same Argon2 parameters as production hashes, so the decoy verify takes the same
// time as a real one.
static DECOY_PASSWORD_HASH: LazyLock<HashedPassword> = LazyLock::new(|| {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(15000, 2, 1, None).expect("decoy Argon2 params valid");
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(b"decoy-password-not-real", &salt)
        .expect("decoy hash computes");
    let hash: Secret = hash.to_string().into();
    HashedPassword::parse_password_hash(&hash).expect("decoy hash parses")
});

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&self, user: User) -> UserStoreResult<()>;
    async fn get_user(&self, email: &Email) -> UserStoreResult<User>;
    async fn validate_user(&self, email: &Email, password: &Secret) -> UserStoreResult<()> {
        match self.get_user(email).await {
            Ok(user) => match user.password.verify_password(password).await {
                Ok(()) => Ok(()),
                Err(_) => Err(UserStoreError::IncorrectCredentials),
            },
            Err(UserStoreError::UserNotFound) => {
                // Run a decoy verify so the latency of this branch matches the
                // "user exists, wrong password" branch and the response time can't be
                // used to enumerate registered emails.
                let _ = DECOY_PASSWORD_HASH.verify_password(password).await;
                Err(UserStoreError::IncorrectCredentials)
            }
            Err(error) => Err(error),
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

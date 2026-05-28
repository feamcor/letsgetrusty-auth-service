use crate::domain::Email;
use crate::domain::HashedPassword;
use crate::domain::Secret;
use crate::domain::User;
use crate::domain::compute_password_hash_sync;
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
// with the same Argon2 parameters as production hashes (via compute_password_hash_sync),
// so the decoy verify takes the same time as a real one — and so any future parameter change
// in domain/password.rs automatically applies here.
//
// Note: this LazyLock initializer runs `compute_password_hash_sync` (~50-100 ms blocking work).
// Call `warm_decoy_password_hash()` from an off-runtime context at startup to avoid blocking
// a tokio worker on the first unknown-email login.
static DECOY_PASSWORD_HASH: LazyLock<HashedPassword> = LazyLock::new(|| {
    let hash_string = compute_password_hash_sync(b"decoy-password-not-real").expect("decoy hash computes");
    let hash: Secret = hash_string.into();
    HashedPassword::parse_password_hash(&hash).expect("decoy hash parses")
});

/// Force-initialize the decoy hash off the tokio runtime so the first login that hits the
/// UserNotFound branch doesn't pay the Argon2id cost on a worker thread. Call once from
/// startup, ideally wrapped in `tokio::task::spawn_blocking`.
pub fn warm_decoy_password_hash() {
    let _ = &*DECOY_PASSWORD_HASH;
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::HashmapUserStore;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use std::time::Duration;
    use std::time::Instant;

    async fn elapsed_for<F, Fut>(op: F) -> Duration
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let started = Instant::now();
        op().await;
        started.elapsed()
    }

    #[tokio::test]
    async fn validate_user_unknown_email_returns_incorrect_credentials() {
        let store = HashmapUserStore::default();
        let email = Email::parse(&SafeEmail().fake::<String>().into()).unwrap();
        let result = store.validate_user(&email, &"AnyPassword123!".into()).await;
        assert!(matches!(result, Err(UserStoreError::IncorrectCredentials)));
    }

    /// Timing-oracle defense: the unknown-email branch must take roughly as long as the
    /// known-email-wrong-password branch. With Argon2id m=19456, both branches spend tens of
    /// milliseconds in the hash; an undefended branch would return in microseconds.
    #[tokio::test]
    async fn validate_user_unknown_branch_is_not_dramatically_faster() {
        let store = HashmapUserStore::default();
        let known_email = Email::parse(&SafeEmail().fake::<String>().into()).unwrap();
        let password: Secret = "CorrectHorseBatteryStaple123!".into();
        let user = User::new(
            &known_email,
            &HashedPassword::parse(&password, &known_email).await.unwrap(),
            false,
        );
        store.add_user(user).await.unwrap();

        // Warm up — Argon2 setup + LazyLock init shouldn't skew the first measurement.
        let _ = store.validate_user(&known_email, &"wrong-Wrong-PASSWORD-111!".into()).await;

        let wrong_password_time = elapsed_for(|| async {
            let _ = store.validate_user(&known_email, &"wrong-Wrong-PASSWORD-111!".into()).await;
        })
        .await;
        let unknown_email = Email::parse(&SafeEmail().fake::<String>().into()).unwrap();
        let unknown_email_time = elapsed_for(|| async {
            let _ = store.validate_user(&unknown_email, &"wrong-Wrong-PASSWORD-111!".into()).await;
        })
        .await;

        // The unknown-email branch must be at least 25% of the wrong-password branch — the
        // pre-fix value was effectively 0%. We don't enforce equality because Argon2 timings
        // vary across runs and CI hardware.
        assert!(
            unknown_email_time * 4 >= wrong_password_time,
            "decoy too cheap: unknown={unknown_email_time:?} wrong={wrong_password_time:?}",
        );
    }
}

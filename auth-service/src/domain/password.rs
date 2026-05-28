use crate::domain::Email;
use crate::domain::Secret;
use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use argon2::Version;
use argon2::password_hash;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use tokio::task::spawn_blocking;
use zxcvbn::Score;
use zxcvbn::zxcvbn;

// NIST Special Publication 800-63B
// Section 3.1.1.2 Password Verifiers
// https://pages.nist.gov/800-63-4/sp800-63b.html
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 64;
pub const PASSWORD_LENGTH_RANGE: std::ops::Range<usize> = MIN_PASSWORD_LENGTH..MAX_PASSWORD_LENGTH + 1;
pub const SAFE_PASSWORD_LENGTH_RANGE: std::ops::Range<usize> = MIN_PASSWORD_LENGTH * 2..MAX_PASSWORD_LENGTH + 1;
const MIN_PASSWORD_ENTROPY: Score = Score::Three;

// OWASP Password Storage Cheat Sheet (Argon2id, t=2, p=1): m >= 19 MiB.
// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
pub const ARGON2_MEMORY_KIB: u32 = 19456;
pub const ARGON2_ITERATIONS: u32 = 2;
pub const ARGON2_PARALLELISM: u32 = 1;

#[derive(thiserror::Error, Debug)]
pub enum PasswordError {
    #[error("Password is too short (min length is {MIN_PASSWORD_LENGTH})")]
    TooShort,
    #[error("Password is too long (max length is {MAX_PASSWORD_LENGTH})")]
    TooLong,
    #[error("Password is weak")]
    Weak,
    #[error(transparent)]
    InvalidPasswordHash(#[from] password_hash::Error),
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type PasswordResult<T> = Result<T, PasswordError>;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HashedPassword(Secret);

impl HashedPassword {
    #[tracing::instrument(name = "HashedPasswordParsing", level = tracing::Level::TRACE, skip_all
    )]
    pub async fn parse(password: &Secret, user: &Email) -> PasswordResult<Self> {
        validate_password_strength(password, user)?;
        let password_hash = compute_password_hash(password).await?;
        Ok(Self(password_hash))
    }

    pub fn parse_password_hash(hash: &Secret) -> PasswordResult<Self> {
        let hash = hash.expose();
        match PasswordHash::new(hash) {
            Ok(password_hash) => {
                let secret = password_hash.to_string().into();
                let hashed_password = Self(secret);
                Ok(hashed_password)
            }
            Err(error) => Err(PasswordError::InvalidPasswordHash(error)),
        }
    }

    #[must_use]
    pub fn as_secret(&self) -> &Secret {
        &self.0
    }

    #[tracing::instrument(name = "RawPasswordVerification", level = tracing::Level::TRACE, skip_all
    )]
    pub async fn verify_password(&self, candidate: &Secret) -> PasswordResult<()> {
        let current_span = tracing::Span::current();
        let current_hash = self.as_secret().expose().to_owned();
        let candidate = candidate.expose().to_owned();
        spawn_blocking(move || {
            current_span.in_scope(|| {
                let expected_hash = PasswordHash::new(&current_hash)?;
                Argon2::default().verify_password(candidate.as_bytes(), &expected_hash)
            })
        })
        .await
        .map_err(|error| {
            tracing::error!("{}", error);
            PasswordError::UnexpectedError(error.into())
        })?
        .map_err(|_| PasswordError::PasswordMismatch)
    }
}

impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        self.as_secret() == other.as_secret()
    }
}

impl Eq for HashedPassword {}

impl std::hash::Hash for HashedPassword {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_secret().hash(state);
    }
}

/// Cheap structural validation: length bounds + zxcvbn entropy. Used by both signup (before the
/// expensive hash) and login (to reject obviously bogus input without doing any Argon2 work).
pub fn validate_password_strength(password: &Secret, user: &Email) -> PasswordResult<()> {
    let raw_password = password.expose();
    if raw_password.len() < MIN_PASSWORD_LENGTH {
        return Err(PasswordError::TooShort);
    }
    if raw_password.len() > MAX_PASSWORD_LENGTH {
        return Err(PasswordError::TooLong);
    }
    let entropy = zxcvbn(raw_password, &[user.as_secret().expose()]);
    // Score 3 means the password can be cracked with 10^10 guesses or fewer.
    if entropy.score() < MIN_PASSWORD_ENTROPY {
        return Err(PasswordError::Weak);
    }
    Ok(())
}

#[tracing::instrument(name = "PasswordHashComputation", level = tracing::Level::TRACE, skip_all
)]
async fn compute_password_hash(password: &Secret) -> color_eyre::eyre::Result<Secret> {
    let current_span = tracing::Span::current();
    let password = password.expose().to_owned();
    spawn_blocking(move || -> color_eyre::eyre::Result<String> {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None)?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let hash = argon2.hash_password(password.as_bytes(), &salt)?;
            Ok(hash.to_string())
        })
    })
    .await?
    .map(Secret::from)
    .map_err(|error| {
        tracing::error!("{}", error);
        error
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use quickcheck::Gen;
    use quickcheck_macros::quickcheck;

    const VALID_PASSWORD: &str = "CorrectHorseBatteryStaple123!";

    #[tokio::test]
    async fn test_password_too_short() {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        let password = "a".repeat(MIN_PASSWORD_LENGTH - 1).into();
        let result = HashedPassword::parse(&password, &user).await;
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[tokio::test]
    async fn test_password_too_long() {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        let password = "a".repeat(MAX_PASSWORD_LENGTH + 1).into();
        let result = HashedPassword::parse(&password, &user).await;
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[tokio::test]
    async fn test_password_weak() {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        assert!(matches!(
            HashedPassword::parse(&"password123".into(), &user).await,
            Err(PasswordError::Weak)
        ));
        assert!(matches!(
            HashedPassword::parse(&"12345678".into(), &user).await,
            Err(PasswordError::Weak)
        ));
        assert!(matches!(
            HashedPassword::parse(&"qwertyuiop".into(), &user).await,
            Err(PasswordError::Weak)
        ));
    }

    #[tokio::test]
    async fn test_password_valid() {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        let password = VALID_PASSWORD.into();
        let result = HashedPassword::parse(&password, &user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_exposure() {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        let unhashed = VALID_PASSWORD.into();
        let hashed = HashedPassword::parse(&unhashed, &user).await.unwrap();
        assert_ne!(hashed.as_secret().expose(), unhashed.expose());
    }

    #[test]
    fn can_parse_valid_argon2_hash() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None).unwrap(),
        );
        let hash = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let hash = hash.into();
        let hashed = HashedPassword::parse_password_hash(&hash).unwrap();
        assert_eq!(hashed.as_secret().expose(), hash.expose());
        assert!(hashed.as_secret().expose().starts_with("$argon2id$v=19$"));
    }

    #[tokio::test]
    async fn can_verify_raw_password() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_PARALLELISM, None).unwrap(),
        );
        let hash = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let hash = hash.into();
        let hashed = HashedPassword::parse_password_hash(&hash).unwrap();
        assert_eq!(hashed.as_secret().expose(), hash.expose());
        assert!(hashed.as_secret().expose().starts_with("$argon2id$v=19$"));
        assert!(hashed.verify_password(&raw_password.into()).await.is_ok());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(_: &mut Gen) -> Self {
            let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
            Self(password)
        }
    }

    #[tokio::test]
    #[ignore = "Slow Property Test"]
    #[quickcheck]
    #[allow(clippy::needless_pass_by_value)]
    async fn prop_valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        let email = SafeEmail().fake::<String>().into();
        let user = Email::parse(&email).unwrap();
        let password = valid_password.0.into();
        HashedPassword::parse(&password, &user).await.is_ok()
    }
}

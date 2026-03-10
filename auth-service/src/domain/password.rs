use argon2::Algorithm;
use argon2::Argon2;
use argon2::Params;
use argon2::PasswordHash;
use argon2::PasswordHasher;
use argon2::PasswordVerifier;
use argon2::Version;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use tokio::task::spawn_blocking;
use tracing::error;
use zxcvbn::Score;
use zxcvbn::zxcvbn;

#[allow(unused_imports)]
use tracing::Level;

// NIST Special Publication 800-63B
// Section 3.1.1.2 Password Verifiers
// https://pages.nist.gov/800-63-4/sp800-63b.html
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MAX_PASSWORD_LENGTH: usize = 64;
pub const PASSWORD_LENGTH_RANGE: std::ops::Range<usize> =
    MIN_PASSWORD_LENGTH..MAX_PASSWORD_LENGTH + 1;
pub const SAFE_PASSWORD_LENGTH_RANGE: std::ops::Range<usize> =
    MIN_PASSWORD_LENGTH * 2..MAX_PASSWORD_LENGTH + 1;
const MIN_PASSWORD_ENTROPY: Score = Score::Three;

#[derive(thiserror::Error, Debug)]
pub enum PasswordError {
    #[error("Password is too short (min length is {MIN_PASSWORD_LENGTH})")]
    TooShort,
    #[error("Password is too long (max length is {MAX_PASSWORD_LENGTH})")]
    TooLong,
    #[error("Password is weak")]
    Weak,
    #[error("Invalid password hash: {0}")]
    InvalidPasswordHash(String),
    #[error("Password mismatch")]
    PasswordMismatch,
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

#[derive(Debug, Clone)]
pub struct HashedPassword(SecretString);

impl HashedPassword {
    #[tracing::instrument(name = "HashedPasswordParsing", level = Level::TRACE, skip_all)]
    pub async fn parse(raw: &str, user: &str) -> Result<Self, PasswordError> {
        if raw.len() < MIN_PASSWORD_LENGTH {
            return Err(PasswordError::TooShort);
        }
        if raw.len() > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::TooLong);
        }
        let entropy = zxcvbn(raw, &[user]);
        // Score 3 mean that the password can be cracked with 10^10 guesses or fewer.
        if entropy.score() < MIN_PASSWORD_ENTROPY {
            return Err(PasswordError::Weak);
        }
        let hash = compute_password_hash(raw).await?;
        let secret = SecretString::from(hash);
        Ok(Self(secret))
    }

    pub fn parse_password_hash(hash: &str) -> Result<Self, PasswordError> {
        match PasswordHash::new(hash) {
            Ok(password_hash) => {
                let secret = SecretString::from(password_hash.to_string());
                let hashed_password = Self(secret);
                Ok(hashed_password)
            }
            Err(error) => Err(PasswordError::InvalidPasswordHash(error.to_string())),
        }
    }

    #[tracing::instrument(name = "RawPasswordVerification", level = Level::TRACE, skip_all)]
    pub async fn verify_raw_password(&self, candidate: &str) -> Result<(), PasswordError> {
        let current_span = tracing::Span::current();
        let candidate = candidate.to_owned();
        let secret = self.0.expose_secret().to_owned();
        let task = spawn_blocking(move || {
            current_span.in_scope(|| {
                let expected_hash = PasswordHash::new(&secret)?;
                Argon2::default().verify_password(candidate.as_bytes(), &expected_hash)
            })
        });
        match task.await {
            Ok(result) => match result {
                Ok(()) => Ok(()),
                Err(_) => Err(PasswordError::PasswordMismatch),
            },
            Err(error) => {
                error!("{}", error);
                Err(PasswordError::Unexpected(error.to_string()))
            }
        }
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        self.0.expose_secret()
    }
}

#[tracing::instrument(name = "PasswordHashComputation", level = Level::TRACE, skip_all)]
async fn compute_password_hash(password: &str) -> Result<String, PasswordError> {
    let current_span = tracing::Span::current();
    let password = password.to_owned();
    let task = spawn_blocking(move || -> Result<String, PasswordError> {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut OsRng);
            let params = Params::new(15000, 2, 1, None)
                .map_err(|e| PasswordError::Unexpected(e.to_string()))?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| PasswordError::Unexpected(e.to_string()))?;
            Ok(hash.to_string())
        })
    });
    task.await.unwrap_or_else(|error| {
        error!("{}", error);
        Err(PasswordError::Unexpected(error.to_string()))
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
        let user: String = SafeEmail().fake();
        let password = "a".repeat(MIN_PASSWORD_LENGTH - 1);
        let result = HashedPassword::parse(&password, &user).await;
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[tokio::test]
    async fn test_password_too_long() {
        let user: String = SafeEmail().fake();
        let password = "a".repeat(MAX_PASSWORD_LENGTH + 1);
        let result = HashedPassword::parse(&password, &user).await;
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[tokio::test]
    async fn test_password_weak() {
        let user: String = SafeEmail().fake();
        assert!(matches!(
            HashedPassword::parse("password123", &user).await,
            Err(PasswordError::Weak)
        ));
        assert!(matches!(
            HashedPassword::parse("12345678", &user).await,
            Err(PasswordError::Weak)
        ));
        assert!(matches!(
            HashedPassword::parse("qwertyuiop", &user).await,
            Err(PasswordError::Weak)
        ));
    }

    #[tokio::test]
    async fn test_password_valid() {
        let user: String = SafeEmail().fake();
        let password = VALID_PASSWORD;
        let result = HashedPassword::parse(password, &user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_password_exposure() {
        let user: String = SafeEmail().fake();
        let password_str = VALID_PASSWORD;
        let password = HashedPassword::parse(password_str, &user).await.unwrap();
        assert_ne!(password.expose(), password_str);
    }

    #[test]
    fn can_parse_valid_argon2_hash() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );
        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let hash_password = HashedPassword::parse_password_hash(&hash_string).unwrap();
        assert_eq!(hash_password.as_ref(), hash_string.as_str());
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));
    }

    #[tokio::test]
    async fn can_verify_raw_password() {
        let raw_password = "TestPassword123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );
        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let hash_password = HashedPassword::parse_password_hash(&hash_string).unwrap();
        assert_eq!(hash_password.0.expose_secret(), hash_string.as_str());
        assert!(
            hash_password
                .0
                .expose_secret()
                .starts_with("$argon2id$v=19$")
        );
        assert!(
            hash_password
                .verify_raw_password(raw_password)
                .await
                .is_ok()
        );
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(_: &mut Gen) -> Self {
            let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
            Self(password)
        }
    }

    #[tokio::test]
    #[ignore]
    #[quickcheck]
    async fn prop_valid_passwords_are_parsed_successfully(
        valid_password: ValidPasswordFixture,
    ) -> bool {
        let user: String = SafeEmail().fake();
        HashedPassword::parse(&valid_password.0, &user)
            .await
            .is_ok()
    }
}

use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;
use zxcvbn::{zxcvbn, Score};

// NIST Special Publication 800-63B
// Section 3.1.1.2 Password Verifiers
// https://pages.nist.gov/800-63-4/sp800-63b.html
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 64;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password is too short (min length is {MIN_PASSWORD_LENGTH})")]
    TooShort,
    #[error("Password is too long (max length is {MAX_PASSWORD_LENGTH})")]
    TooLong,
    #[error("Password is weak")]
    Weak,
}

#[derive(Debug, Clone)]
pub struct Password(SecretString);

impl Password {
    pub fn parse(raw: &str, user: &str) -> Result<Self, PasswordError> {
        if raw.len() < MIN_PASSWORD_LENGTH {
            return Err(PasswordError::TooShort);
        }
        if raw.len() > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::TooLong);
        }

        let entropy = zxcvbn(raw, &[user]);
        // Score 3 mean that the password can be cracked with 10^10 guesses or fewer.
        if entropy.score() < Score::Three {
            return Err(PasswordError::Weak);
        }

        Ok(Self(SecretString::from(raw)))
    }

    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck_macros::quickcheck;

    const VALID_PASSWORD: &str = "CorrectHorseBatteryStaple123!";

    #[test]
    fn test_password_too_short() {
        let user: String = SafeEmail().fake();
        let password = "a".repeat(MIN_PASSWORD_LENGTH - 1);
        let result = Password::parse(&password, &user);
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_password_too_long() {
        let user: String = SafeEmail().fake();
        let password = "a".repeat(MAX_PASSWORD_LENGTH + 1);
        let result = Password::parse(&password, &user);
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[test]
    fn test_password_weak() {
        let user: String = SafeEmail().fake();
        assert!(matches!(Password::parse("password123", &user), Err(PasswordError::Weak)));
        assert!(matches!(Password::parse("12345678", &user), Err(PasswordError::Weak)));
        assert!(matches!(Password::parse("qwertyuiop", &user), Err(PasswordError::Weak)));
    }

    #[test]
    fn test_password_valid() {
        let user: String = SafeEmail().fake();
        let password = VALID_PASSWORD;
        let result = Password::parse(password, &user);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_exposure() {
        let user: String = SafeEmail().fake();
        let password_str = VALID_PASSWORD;
        let password = Password::parse(password_str, &user).unwrap();
        assert_eq!(password.expose(), password_str);
    }

    #[quickcheck]
    fn prop_password_parse_never_panics(password: String) -> bool {
        let user: String = SafeEmail().fake();
        let _ = Password::parse(&password, &user);
        true
    }
}
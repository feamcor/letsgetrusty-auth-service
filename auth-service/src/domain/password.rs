use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;

#[derive(Debug, Default)]
struct Metrics {
    upper: usize,
    lower: usize,
    digits: usize,
    special: usize,
    invalid: usize,
}

const SPECIAL_CHARS: &str = "!@#$%&*-_=+";
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 64;
const MIN_SPECIAL_CHARS: usize = 1;
const MIN_UPPERCASE_CHARS: usize = 2;
const MIN_LOWERCASE_CHARS: usize = 2;
const MIN_DIGITS: usize = 2;

fn analyze_password(password: &str) -> Metrics {
    let mut metrics = Metrics::default();
    for c in password.chars() {
        if c.is_ascii_uppercase() {
            metrics.upper += 1;
        } else if c.is_ascii_lowercase() {
            metrics.lower += 1;
        } else if c.is_ascii_digit() {
            metrics.digits += 1;
        } else if SPECIAL_CHARS.contains(c) {
            metrics.special += 1;
        } else {
            metrics.invalid += 1;
        }
    }
    metrics
}

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("Password is too short (min length is {MIN_PASSWORD_LENGTH})")]
    TooShort,
    #[error("Password is too long (max length is {MAX_PASSWORD_LENGTH})")]
    TooLong,
    #[error("Password is weak")]
    Weak,
    #[error("Password contains invalid characters")]
    InvalidChars,
}

#[derive(Debug, Clone)]
pub struct Password(SecretString);

impl Password {
    pub fn parse(raw: &str) -> Result<Self, PasswordError> {
        if raw.len() < MIN_PASSWORD_LENGTH {
            return Err(PasswordError::TooShort);
        }
        if raw.len() > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::TooLong);
        }
        let metrics = analyze_password(raw);
        if metrics.invalid > 0 {
            return Err(PasswordError::InvalidChars);
        }
        if metrics.upper < MIN_UPPERCASE_CHARS ||
            metrics.lower < MIN_LOWERCASE_CHARS ||
            metrics.digits < MIN_DIGITS ||
            metrics.special < MIN_SPECIAL_CHARS
        {
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

    #[test]
    fn test_password_validation() {
        assert!(Password::parse("StrongPassword123!").is_ok());
        assert!(Password::parse("^_Strong( Password )123_$").is_err());
        assert!(Password::parse("PaSs12!!^").is_err());
        assert!(Password::parse("PaSs 12!!").is_err());
        assert!(Password::parse("PaSs12! ").is_err());
        assert!(Password::parse(" PaSs12!").is_err());
        assert!(Password::parse("1234567").is_err());
        assert!(Password::parse("12345678901234567890123456789012345678901234567890123456789012345").is_err());
        assert!(Password::parse("Weak123!").is_err());
        assert!(Password::parse("WEAk123!").is_err());
        assert!(Password::parse("WeaK1__!").is_err());
        assert!(Password::parse("WeaK1234").is_err());
    }

    #[test]
    fn test_password_exposure() {
        let password = Password::parse("StrongPassword123!").unwrap();
        assert_eq!(password.expose(), "StrongPassword123!");
    }
}
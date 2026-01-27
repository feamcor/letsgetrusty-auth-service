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
    use quickcheck_macros::quickcheck;

    const VALID_PASSWORD: &str = "StrongPassword123!";

    #[test]
    fn test_password_too_short() {
        let password = "a".repeat(MIN_PASSWORD_LENGTH - 1);
        let result = Password::parse(&password);
        assert!(matches!(result, Err(PasswordError::TooShort)));
    }

    #[test]
    fn test_password_too_long() {
        let password = "a".repeat(MAX_PASSWORD_LENGTH + 1);
        let result = Password::parse(&password);
        assert!(matches!(result, Err(PasswordError::TooLong)));
    }

    #[test]
    fn test_password_invalid_chars() {
        let password = "StrongPassword123! ";
        let result = Password::parse(password);
        assert!(matches!(result, Err(PasswordError::InvalidChars)));

        let password = "StrongPassword123!\t";
        let result = Password::parse(password);
        assert!(matches!(result, Err(PasswordError::InvalidChars)));
    }

    #[test]
    fn test_password_weak() {
        // Missing uppercase
        assert!(matches!(Password::parse("strongpassword123!"), Err(PasswordError::Weak)));
        // Missing lowercase
        assert!(matches!(Password::parse("STRONGPASSWORD123!"), Err(PasswordError::Weak)));
        // Missing digits
        assert!(matches!(Password::parse("StrongPassword!!!"), Err(PasswordError::Weak)));
        // Missing special chars
        assert!(matches!(Password::parse("StrongPassword123"), Err(PasswordError::Weak)));
    }

    #[test]
    fn test_password_valid() {
        let password = VALID_PASSWORD;
        let result = Password::parse(password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_exposure() {
        let password_str = VALID_PASSWORD;
        let password = Password::parse(password_str).unwrap();
        assert_eq!(password.expose(), password_str);
    }

    #[quickcheck]
    fn prop_password_parse_never_panics(password: String) -> bool {
        let _ = Password::parse(&password);
        true
    }

    #[quickcheck]
    fn prop_valid_passwords_are_accepted(
        upper: u8,
        lower: u8,
        digits: u8,
        special: u8,
    ) -> bool {
        let upper_count = (upper as usize % 5) + MIN_UPPERCASE_CHARS;
        let lower_count = (lower as usize % 5) + MIN_LOWERCASE_CHARS;
        let digits_count = (digits as usize % 5) + MIN_DIGITS;
        let special_count = (special as usize % 5) + MIN_SPECIAL_CHARS;

        let mut password_chars: Vec<char> = Vec::new();
        password_chars.extend(std::iter::repeat('A').take(upper_count));
        password_chars.extend(std::iter::repeat('a').take(lower_count));
        password_chars.extend(std::iter::repeat('1').take(digits_count));
        
        let specials: Vec<char> = SPECIAL_CHARS.chars().collect();
        password_chars.extend(std::iter::repeat(specials[special as usize % specials.len()]).take(special_count));

        while password_chars.len() < MIN_PASSWORD_LENGTH {
            password_chars.push('A');
        }

        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        password_chars.shuffle(&mut rng);

        let password: String = password_chars.into_iter().collect();

        if password.len() > MAX_PASSWORD_LENGTH {
            return true;
        }

        let result = Password::parse(&password);
        result.is_ok()
    }
}
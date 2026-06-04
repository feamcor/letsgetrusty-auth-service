use crate::domain::Email;
use crate::domain::HashedPassword;

/// An authenticated account: a validated [`Email`], an Argon2 [`HashedPassword`], and the
/// account's two-factor-authentication preference.
#[derive(Debug, Clone)]
pub struct User {
    pub email: Email,
    pub password: HashedPassword,
    pub requires_2fa: bool,
}

impl User {
    /// Assemble a [`User`] from already-validated components.
    #[must_use]
    pub fn new(email: &Email, password: &HashedPassword, requires_2fa: bool) -> Self {
        Self {
            email: email.clone(),
            password: password.clone(),
            requires_2fa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SAFE_PASSWORD_LENGTH_RANGE;
    use fake::Fake;
    use fake::faker::internet::en::DomainSuffix;
    use fake::faker::internet::en::SafeEmail;
    use fake::rand;

    #[tokio::test]
    async fn should_return_ok_for_valid_input() {
        let raw_email = SafeEmail().fake::<String>().into();
        let email = Email::parse(&raw_email).unwrap();
        let raw_password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().into();
        let password = HashedPassword::parse(&raw_password, &email).await.unwrap();
        let requires_2fa = rand::random();
        let _ = User::new(&email, &password, requires_2fa);
    }

    #[tokio::test]
    async fn should_return_error_for_empty_email() {
        let raw_email = String::new().into();
        let email = Email::parse(&raw_email);
        assert!(email.is_err());
    }

    #[tokio::test]
    async fn should_return_error_for_empty_password() {
        let raw_email = SafeEmail().fake::<String>().into();
        let email = Email::parse(&raw_email).unwrap();
        let raw_password = String::new().into();
        let password = HashedPassword::parse(&raw_password, &email).await;
        assert!(password.is_err());
    }

    #[tokio::test]
    async fn should_return_invalid_email_error() {
        let raw_email = DomainSuffix().fake::<String>().into();
        let email = Email::parse(&raw_email);
        assert!(email.is_err());
    }

    #[tokio::test]
    async fn should_return_invalid_password_error() {
        let raw_email = SafeEmail().fake::<String>().into();
        let email = Email::parse(&raw_email).unwrap();
        let raw_password = (1..7).fake::<String>().into();
        let password = HashedPassword::parse(&raw_password, &email).await;
        assert!(password.is_err());
    }
}

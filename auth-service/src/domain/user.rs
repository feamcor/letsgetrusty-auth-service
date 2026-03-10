use crate::domain::Email;
use crate::domain::EmailError;
use crate::domain::HashedPassword;
use crate::domain::PasswordError;

#[derive(Debug, Clone)]
pub struct User {
    pub email: Email,
    pub password: HashedPassword,
    pub requires_2fa: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum UserError {
    #[error(transparent)]
    InvalidEmail(#[from] EmailError),
    #[error("Invalid password: {0}")]
    InvalidPassword(PasswordError),
}

impl User {
    pub async fn try_new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, UserError> {
        let email = Email::parse(email).map_err(UserError::InvalidEmail)?;
        let password = match HashedPassword::parse(password, email.as_ref()).await {
            Ok(password) => password,
            Err(error) => return Err(UserError::InvalidPassword(error)),
        };
        Ok(Self {
            email,
            password,
            requires_2fa,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SAFE_PASSWORD_LENGTH_RANGE;
    use fake::Fake;
    use fake::faker::internet::en::DomainSuffix;
    use fake::faker::internet::en::Password;
    use fake::faker::internet::en::SafeEmail;
    use fake::rand;

    #[tokio::test]
    async fn should_return_ok_for_valid_input() {
        let email: String = SafeEmail().fake();
        let password: String = SAFE_PASSWORD_LENGTH_RANGE.fake();
        let requires_2fa = rand::random();
        let result = User::try_new(&email, &password, requires_2fa).await;
        assert!(result.is_ok(), "Failed for email: {email} and password: {password}");
    }

    #[tokio::test]
    async fn should_return_error_for_empty_email() {
        let email = "";
        let password: String = Password(16..64).fake();
        let requires_2fa = rand::random();
        let result = User::try_new(email, &password, requires_2fa).await;
        assert!(matches!(result, Err(UserError::InvalidEmail(_))));
    }

    #[tokio::test]
    async fn should_return_error_for_empty_password() {
        let email: String = SafeEmail().fake();
        let password = "";
        let requires_2fa = rand::random();
        let result = User::try_new(&email, password, requires_2fa).await;
        assert!(matches!(result, Err(UserError::InvalidPassword(_))));
    }

    #[tokio::test]
    async fn should_return_invalid_email_error() {
        let email: String = DomainSuffix().fake();
        let password: String = SAFE_PASSWORD_LENGTH_RANGE.fake();
        let requires_2fa = rand::random();
        let result = User::try_new(&email, &password, requires_2fa).await;
        assert!(matches!(result, Err(UserError::InvalidEmail(_))));
    }

    #[tokio::test]
    async fn should_return_invalid_password_error() {
        let email: String = SafeEmail().fake();
        let password: String = Password(1..7).fake();
        let requires_2fa = rand::random();
        let result = User::try_new(&email, &password, requires_2fa).await;
        assert!(matches!(result, Err(UserError::InvalidPassword(_))));
    }
}

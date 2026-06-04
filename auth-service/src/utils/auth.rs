use crate::domain::Email;
use crate::domain::Secret;
use crate::domain::Token;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use chrono::Utc;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::encode;

/// Name of the cookie carrying the session JWT.
pub const JWT_COOKIE_NAME: &str = "jwt";

/// Convenience alias for a fallible token-generation operation.
pub type GenerateTokenResult<T> = Result<T, GenerateTokenError>;

/// Failure modes of JWT minting (signing or expiration arithmetic).
#[derive(thiserror::Error, Debug)]
pub enum GenerateTokenError {
    #[error(transparent)]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Expiration delta is out-of-range")]
    ExpirationDeltaOutOfRange,
    #[error("Expiration is out-of-range")]
    ExpirationOutOfRange,
    #[error("Expiration conversion error")]
    ExpirationConversionError,
}

/// Mint a signed session JWT for `email` and wrap it in a hardened auth cookie.
///
/// # Errors
///
/// Returns [`GenerateTokenError`] if `ttl` is out of range or JWT signing fails.
pub fn generate_auth_cookie(email: &Email, secret: &Secret, ttl: i64) -> GenerateTokenResult<Cookie<'static>> {
    let token = generate_auth_token(email, secret, ttl)?;
    Ok(create_auth_cookie(&token))
}

/// Wrap an existing token in the hardened auth cookie (`HttpOnly`, `Secure`, `SameSite=Lax`).
#[must_use]
pub fn create_auth_cookie(token: &Token) -> Cookie<'static> {
    Cookie::build((JWT_COOKIE_NAME, token.as_secret().expose().to_owned()))
        .http_only(true) // prevent JavaScript from accessing the cookie
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .secure(true)
        .path("/") // apply cookie to all URLs on the server
        .build()
}

fn generate_auth_token(email: &Email, secret: &Secret, ttl: i64) -> GenerateTokenResult<Token> {
    let delta = chrono::Duration::try_seconds(ttl).ok_or(GenerateTokenError::ExpirationDeltaOutOfRange)?;
    let expiration = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::ExpirationOutOfRange)?
        .timestamp();
    let expiration: usize = expiration
        .try_into()
        .map_err(|_| GenerateTokenError::ExpirationConversionError)?;
    let claims = Claims {
        sub: email.as_secret().expose().to_owned(),
        exp: expiration,
    };
    let token = create_token(&claims, secret).map_err(GenerateTokenError::TokenError)?;
    Ok(token)
}

/// JWT claims: `sub` is the account email, `exp` the Unix expiry timestamp.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

/// Decode and verify a session JWT against `secret`, returning its [`Claims`].
///
/// # Errors
///
/// Returns a [`jsonwebtoken`] error if the signature is invalid, the token is malformed, or it
/// has expired.
pub fn validate_token(token: &Token, secret: &Secret) -> jsonwebtoken::errors::Result<Claims> {
    decode::<Claims>(
        token.as_secret().expose(),
        &DecodingKey::from_secret(secret.expose().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

fn create_token(claims: &Claims, secret: &Secret) -> jsonwebtoken::errors::Result<Token> {
    let raw_token = encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.expose().as_bytes()),
    )?;
    Ok(Token::new(&raw_token.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>().into();
        let parsed_email = Email::parse(&email).unwrap();
        let secret = config::secret_from_environment(config::jwt::var::SECRET);
        let cookie = generate_auth_cookie(&parsed_email, &secret, i64::from(config::jwt::default::TTL)).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        dotenvy::dotenv_override().ok();
        let token = Token::new(&"test_token".into());
        let cookie = create_auth_cookie(&token);
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token.as_secret().expose());
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>().into();
        let parsed_email = Email::parse(&email).unwrap();
        let secret = config::secret_from_environment(config::jwt::var::SECRET);
        let token = generate_auth_token(&parsed_email, &secret, i64::from(config::jwt::default::TTL)).unwrap();
        assert_eq!(token.as_secret().expose().split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>().into();
        let parsed_email = Email::parse(&email).unwrap();
        let secret = config::secret_from_environment(config::jwt::var::SECRET);
        let token = generate_auth_token(&parsed_email, &secret, i64::from(config::jwt::default::TTL)).unwrap();
        let result = validate_token(&token, &config::secret_from_environment(config::jwt::var::SECRET)).unwrap();
        assert_eq!(result.sub, email.expose());
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();
        assert!(result.exp > usize::try_from(expiration).expect("valid expiration timestamp"));
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        dotenvy::dotenv_override().ok();
        let token = Token::new(&"invalid_token".into());
        let secret = config::secret_from_environment(config::jwt::var::SECRET);
        let result = validate_token(&token, &secret);
        assert!(result.is_err());
    }
}

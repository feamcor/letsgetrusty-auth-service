use crate::domain::Email;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use chrono::Utc;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Validation;
use jsonwebtoken::decode;
use jsonwebtoken::encode;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use serde::Serialize;

pub const JWT_COOKIE_NAME: &str = "jwt";

pub type GenerateTokenResult<T> = Result<T, GenerateTokenError>;

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

pub fn generate_auth_cookie(email: &Email, secret: &SecretString, ttl: i64) -> GenerateTokenResult<Cookie<'static>> {
    let token = generate_auth_token(email, secret, ttl)?;
    Ok(create_auth_cookie(token))
}

#[must_use]
pub fn create_auth_cookie(token: String) -> Cookie<'static> {
    Cookie::build((JWT_COOKIE_NAME, token))
        .http_only(true) // prevent JavaScript from accessing the cookie
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .secure(true)
        .path("/") // apply cookie to all URLs on the server
        .build()
}

fn generate_auth_token(email: &Email, secret: &SecretString, ttl: i64) -> GenerateTokenResult<String> {
    let delta = chrono::Duration::try_seconds(ttl).ok_or(GenerateTokenError::ExpirationDeltaOutOfRange)?;
    let expiration = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::ExpirationOutOfRange)?
        .timestamp();
    let expiration: usize = expiration
        .try_into()
        .map_err(|_| GenerateTokenError::ExpirationConversionError)?;
    let subscriber = email.as_ref().to_owned();
    let claims = Claims {
        sub: subscriber,
        exp: expiration,
    };
    create_token(&claims, secret).map_err(GenerateTokenError::TokenError)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn validate_token(token: &str, secret: &SecretString) -> jsonwebtoken::errors::Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

fn create_token(claims: &Claims, secret: &SecretString) -> jsonwebtoken::errors::Result<String> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::consts;
    use crate::config::secret_from_environment;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>();
        let email = Email::parse(&email).unwrap();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let cookie =
            generate_auth_cookie(&email, &secret, i64::from(consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT)).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        dotenvy::dotenv_override().ok();
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>();
        let email = Email::parse(&email).unwrap();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let result =
            generate_auth_token(&email, &secret, i64::from(consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT)).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        dotenvy::dotenv_override().ok();
        let email_string = SafeEmail().fake::<String>();
        let email = Email::parse(&email_string).unwrap();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let token =
            generate_auth_token(&email, &secret, i64::from(consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT)).unwrap();
        let result = validate_token(
            &token,
            &secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(result.sub, email_string);
        let expiration = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();
        assert!(result.exp > usize::try_from(expiration).expect("valid expiration timestamp"));
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        dotenvy::dotenv_override().ok();
        let token = "invalid_token".to_owned();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let result = validate_token(&token, &secret).await;
        assert!(result.is_err());
    }
}

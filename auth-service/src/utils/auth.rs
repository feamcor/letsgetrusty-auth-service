use crate::domain::Email;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

pub const JWT_COOKIE_NAME: &str = "jwt";

pub fn generate_auth_cookie(
    email: &Email,
    secret: &SecretString,
    ttl: i64,
) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email, secret, ttl)?;
    Ok(create_auth_cookie(token))
}

pub fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .http_only(true) // prevent JavaScript from accessing the cookie
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations.
        .secure(true)
        .path("/") // apply cookie to all URLs on the server
        .build();
    cookie
}

#[derive(thiserror::Error, Debug)]
pub enum GenerateTokenError {
    #[error("Token error: {0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Unexpected error")]
    UnexpectedError,
}

fn generate_auth_token(
    email: &Email,
    secret: &SecretString,
    ttl: i64,
) -> Result<String, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(ttl).ok_or(GenerateTokenError::UnexpectedError)?;
    let expiration = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();
    let expiration: usize = expiration
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError)?;
    let subscriber = email.as_ref().to_owned();
    let claims = Claims {
        sub: subscriber,
        exp: expiration,
    };
    create_token(&claims, secret).map_err(GenerateTokenError::TokenError)
}

pub async fn validate_token(
    token: &str,
    secret: &SecretString,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

fn create_token(
    claims: &Claims,
    secret: &SecretString,
) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.expose_secret().as_bytes()),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{consts, secret_from_environment};
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        dotenvy::dotenv_override().ok();
        let email = SafeEmail().fake::<String>();
        let email = Email::parse(&email).unwrap();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let cookie = generate_auth_cookie(
            &email,
            &secret,
            consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
        )
        .unwrap();
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
        let result = generate_auth_token(
            &email,
            &secret,
            consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
        )
        .unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        dotenvy::dotenv_override().ok();
        let email_string = SafeEmail().fake::<String>();
        let email = Email::parse(&email_string).unwrap();
        let secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET).unwrap();
        let token = generate_auth_token(
            &email,
            &secret,
            consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
        )
        .unwrap();
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
        assert!(result.exp > expiration as usize);
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

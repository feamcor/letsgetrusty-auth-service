use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStore;
use crate::services::TwoFactorAuthCodeStoreError;
use crate::services::TwoFactorAuthCodeStoreResult;
use redis::AsyncCommands;
use redis::SetExpiry;
use redis::SetOptions;
use std::fmt::Debug;

/// Redis-backed [`TwoFactorAuthCodeStore`] with per-entry expiry (the production cache backend).
pub struct RedisTwoFactorAuthCodeStore {
    connection: redis::aio::MultiplexedConnection,
    tfa_ttl_secs: u64,
}

impl Debug for RedisTwoFactorAuthCodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisTwoFactorAuthCodeStore")
            .field("connection", &"MultiplexedConnection")
            .field("tfa_ttl_secs", &self.tfa_ttl_secs)
            .finish()
    }
}

impl RedisTwoFactorAuthCodeStore {
    pub fn new(connection: redis::aio::MultiplexedConnection, tfa_ttl_secs: u64) -> Self {
        Self {
            connection,
            tfa_ttl_secs,
        }
    }
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for RedisTwoFactorAuthCodeStore {
    #[tracing::instrument(name = "Add2FACodeToCache", level = tracing::Level::TRACE, skip_all)]
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFactorAuthCode,
    ) -> TwoFactorAuthCodeStoreResult<()> {
        let key = get_key(&email);
        let tuple = TwoFactorAuthTuple(
            login_attempt_id.as_secret().expose().to_owned(),
            code.as_secret().expose().to_owned(),
        );
        let serialized =
            serde_json::to_string(&tuple).map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let mut connection = self.connection.clone();
        let options = SetOptions::default().with_expiration(SetExpiry::EX(self.tfa_ttl_secs));
        let result: redis::RedisResult<Option<String>> = connection.set_options(key, serialized, options).await;
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(TwoFactorAuthCodeStoreError::UnexpectedError(error.into())),
        }
    }

    async fn remove_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<()> {
        let key = get_key(email);
        let mut connection = self.connection.clone();
        connection
            .del(key)
            .await
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))
    }

    async fn get_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<(LoginAttemptId, TwoFactorAuthCode)> {
        let key = get_key(email);
        let mut connection = self.connection.clone();
        let value: Option<String> = connection
            .get(key)
            .await
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let value = value.ok_or(TwoFactorAuthCodeStoreError::CodeNotFound)?;
        let tuple: TwoFactorAuthTuple =
            serde_json::from_str(&value).map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let login_attempt_id = LoginAttemptId::parse(&tuple.0.into())
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let code = TwoFactorAuthCode::parse(&tuple.1.into())
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        Ok((login_attempt_id, code))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TwoFactorAuthTuple(String, String);

fn get_key(email: &Email) -> String {
    format!("code:2fa:{}", email.as_secret().expose())
}

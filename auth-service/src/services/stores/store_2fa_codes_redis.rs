use crate::config::consts;
use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStore;
use crate::services::TwoFactorAuthCodeStoreError;
use redis::Commands;
use redis::SetExpiry;
use redis::SetOptions;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use tokio::sync::RwLock;

#[allow(unused_imports)]
use tracing::Level;

pub struct RedisTwoFactorAuthCodeStore {
    connection: RwLock<redis::Connection>,
}

impl Debug for RedisTwoFactorAuthCodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisTwoFactorAuthCodeStore")
            .field("connection", &"RwLock<redis::Connection>")
            .finish()
    }
}

impl RedisTwoFactorAuthCodeStore {
    pub fn new(connection: RwLock<redis::Connection>) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for RedisTwoFactorAuthCodeStore {
    #[tracing::instrument(name = "Add2FACodeToCache", level = Level::TRACE, skip_all)]
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFactorAuthCode,
    ) -> Result<(), TwoFactorAuthCodeStoreError> {
        let ttl = u64::from(consts::AUTH_SERVICE_2FA_TTL_SECONDS_DEFAULT);
        let key = get_key(&email);
        let tuple = TwoFactorAuthTuple(
            login_attempt_id.as_ref().to_string(),
            code.as_ref().to_string(),
        );
        let serialized = serde_json::to_string(&tuple)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let mut connection = self.connection.write().await;
        let options = SetOptions::default().with_expiration(SetExpiry::EX(ttl));
        let result: redis::RedisResult<Option<String>> =
            connection.set_options(key, serialized, options);
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(TwoFactorAuthCodeStoreError::UnexpectedError(error.into())),
        }
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError> {
        let key = get_key(email);
        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFactorAuthCode), TwoFactorAuthCodeStoreError> {
        let key = get_key(email);
        let mut connection = self.connection.write().await;
        let value: String = connection
            .get(key)
            .map_err(|_| TwoFactorAuthCodeStoreError::CodeNotFound)?;
        let tuple: TwoFactorAuthTuple = serde_json::from_str(&value)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let login_attempt_id = LoginAttemptId::parse(tuple.0)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let code = TwoFactorAuthCode::parse(tuple.1)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        Ok((login_attempt_id, code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFactorAuthTuple(pub String, pub String);

fn get_key(email: &Email) -> String {
    format!("code:2fa:{}", email.as_ref())
}

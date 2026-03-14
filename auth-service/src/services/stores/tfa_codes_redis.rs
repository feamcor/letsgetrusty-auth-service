use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStore;
use crate::services::TwoFactorAuthCodeStoreError;
use crate::services::TwoFactorAuthCodeStoreResult;
use redis::Commands;
use redis::SetExpiry;
use redis::SetOptions;
use std::fmt::Debug;
use tokio::sync::RwLock;

#[allow(unused_imports)]
use tracing::Level;

pub struct RedisTwoFactorAuthCodeStore {
    connection: RwLock<redis::Connection>,
    tfa_ttl_secs: u64,
}

impl Debug for RedisTwoFactorAuthCodeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisTwoFactorAuthCodeStore")
            .field("connection", &"RwLock<redis::Connection>")
            .finish()
    }
}

impl RedisTwoFactorAuthCodeStore {
    pub fn new(connection: RwLock<redis::Connection>, tfa_ttl_secs: u64) -> Self {
        Self {
            connection,
            tfa_ttl_secs,
        }
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
    ) -> TwoFactorAuthCodeStoreResult<()> {
        let ttl = u64::from(self.tfa_ttl_secs);
        let key = get_key(&email);
        let tuple = TwoFactorAuthTuple(
            login_attempt_id.as_secret().expose().to_owned(),
            code.as_secret().expose().to_owned(),
        );
        let serialized =
            serde_json::to_string(&tuple).map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))?;
        let mut connection = self.connection.write().await;
        let options = SetOptions::default().with_expiration(SetExpiry::EX(ttl));
        let result: redis::RedisResult<Option<String>> = connection.set_options(key, serialized, options);
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(TwoFactorAuthCodeStoreError::UnexpectedError(error.into())),
        }
    }

    async fn remove_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<()> {
        let key = get_key(email);
        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .map_err(|e| TwoFactorAuthCodeStoreError::UnexpectedError(e.into()))
    }

    async fn get_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<(LoginAttemptId, TwoFactorAuthCode)> {
        let key = get_key(email);
        let mut connection = self.connection.write().await;
        let value: String = connection
            .get(key)
            .map_err(|_| TwoFactorAuthCodeStoreError::CodeNotFound)?;
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
struct TwoFactorAuthTuple(pub String, pub String);

fn get_key(email: &Email) -> String {
    format!("code:2fa:{}", email.as_secret().expose())
}

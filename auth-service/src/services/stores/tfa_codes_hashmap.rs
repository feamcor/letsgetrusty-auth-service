use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStore;
use crate::services::TwoFactorAuthCodeStoreError;
use crate::services::TwoFactorAuthCodeStoreResult;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;

type Entry = (LoginAttemptId, TwoFactorAuthCode, Instant);

#[derive(Debug)]
pub struct HashmapTwoFactorAuthCodeStore {
    codes: RwLock<HashMap<Email, Entry>>,
    ttl: Duration,
}

impl HashmapTwoFactorAuthCodeStore {
    #[must_use]
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            codes: RwLock::default(),
            ttl: Duration::from_secs(ttl_secs),
        }
    }
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for HashmapTwoFactorAuthCodeStore {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFactorAuthCode,
    ) -> TwoFactorAuthCodeStoreResult<()> {
        let mut codes = self.codes.write().await;
        codes.insert(email, (login_attempt_id, two_fa_code, Instant::now()));
        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<()> {
        let mut codes = self.codes.write().await;
        codes.remove(email);
        Ok(())
    }

    async fn get_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<(LoginAttemptId, TwoFactorAuthCode)> {
        // Read under a read-lock first to keep the hot path lock-free of writes.
        {
            let codes = self.codes.read().await;
            if let Some((id, code, stored_at)) = codes.get(email)
                && stored_at.elapsed() < self.ttl
            {
                return Ok((id.clone(), code.clone()));
            }
        }
        // Either absent or expired. Re-check under the write lock before removing: a concurrent
        // add_code between releasing the read lock and acquiring the write lock could have
        // inserted a fresh entry that we must NOT delete.
        let mut codes = self.codes.write().await;
        match codes.get(email) {
            Some((id, code, stored_at)) if stored_at.elapsed() < self.ttl => {
                // A concurrent add_code raced in with a fresh entry — return it instead of sweeping.
                Ok((id.clone(), code.clone()))
            }
            Some(_) => {
                // Confirmed stale under the write lock; sweep and report miss.
                codes.remove(email);
                Err(TwoFactorAuthCodeStoreError::CodeNotFound)
            }
            None => Err(TwoFactorAuthCodeStoreError::CodeNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[tokio::test]
    async fn test_add_get_and_remove_code() {
        let store = HashmapTwoFactorAuthCodeStore::new(300);
        let email = SafeEmail().fake::<String>().into();
        let parsed_email = Email::parse(&email).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFactorAuthCode::default();

        let result = store
            .add_code(parsed_email.clone(), login_attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());

        let (retrieved_id, retrieved_code) = store.get_code(&parsed_email).await.unwrap();
        assert_eq!(retrieved_id, login_attempt_id);
        assert_eq!(retrieved_code, code);

        let result = store.remove_code(&parsed_email).await;
        assert!(result.is_ok());

        let result = store.get_code(&parsed_email).await;
        assert!(matches!(result, Err(TwoFactorAuthCodeStoreError::CodeNotFound)));
    }

    #[tokio::test]
    async fn expired_code_is_reported_as_not_found() {
        // 0-second TTL: any positive elapsed time means the entry has already expired.
        let store = HashmapTwoFactorAuthCodeStore::new(0);
        let email = SafeEmail().fake::<String>().into();
        let parsed_email = Email::parse(&email).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFactorAuthCode::default();

        store
            .add_code(parsed_email.clone(), login_attempt_id, code)
            .await
            .unwrap();
        // Give the monotonic clock a tick so `elapsed()` is strictly > 0.
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result = store.get_code(&parsed_email).await;
        assert!(matches!(result, Err(TwoFactorAuthCodeStoreError::CodeNotFound)));

        // A second read should still return CodeNotFound — the sweep on the first read removed the entry.
        let result = store.get_code(&parsed_email).await;
        assert!(matches!(result, Err(TwoFactorAuthCodeStoreError::CodeNotFound)));
    }
}

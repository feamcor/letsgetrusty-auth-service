use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;
use crate::services::TwoFactorAuthCodeStore;
use crate::services::TwoFactorAuthCodeStoreError;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct HashmapTwoFactorAuthCodeStore {
    codes: RwLock<HashMap<Email, (LoginAttemptId, TwoFactorAuthCode)>>,
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for HashmapTwoFactorAuthCodeStore {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFactorAuthCode,
    ) -> Result<(), TwoFactorAuthCodeStoreError> {
        let mut codes = self.codes.write().await;
        codes.insert(email, (login_attempt_id, two_fa_code));
        Ok(())
    }

    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError> {
        let mut codes = self.codes.write().await;
        codes.remove(email);
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFactorAuthCode), TwoFactorAuthCodeStoreError> {
        let codes = self.codes.read().await;
        match codes.get(email) {
            Some(code) => Ok(code.clone()),
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
        let store = HashmapTwoFactorAuthCodeStore::default();
        let email = Email::parse(SafeEmail().fake::<String>().as_str()).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFactorAuthCode::default();

        let result = store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());

        let (retrieved_id, retrieved_code) = store.get_code(&email).await.unwrap();
        assert_eq!(retrieved_id, login_attempt_id);
        assert_eq!(retrieved_code, code);

        let result = store.remove_code(&email).await;
        assert!(result.is_ok());

        let result = store.get_code(&email).await;
        assert!(matches!(
            result,
            Err(TwoFactorAuthCodeStoreError::CodeNotFound)
        ));
    }
}

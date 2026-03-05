use crate::config::ConfigType;
use crate::services::{
    BannedTokenStoreType, EmailClientType, TwoFactorAuthCodeStoreType, UserStoreType,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
    pub email_client: EmailClientType,
    pub config: ConfigType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
        email_client: EmailClientType,
        config: ConfigType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_factor_auth_code_store,
            email_client,
            config,
        }
    }
}

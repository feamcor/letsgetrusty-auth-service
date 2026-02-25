use crate::services::{HashmapTwoFactorAuthCodeStore, HashmapUserStore, HashsetBannedTokenStore};
use std::sync::Arc;

pub type UserStoreType = Arc<HashmapUserStore>;
pub type BannedTokenStoreType = Arc<HashsetBannedTokenStore>;
pub type TwoFACodeStoreType = Arc<HashmapTwoFactorAuthCodeStore>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
        }
    }
}

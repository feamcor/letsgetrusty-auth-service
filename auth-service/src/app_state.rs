use crate::services::{HashmapUserStore, HashsetBannedTokenStore};
use std::sync::Arc;

pub type UserStoreType = Arc<HashmapUserStore>;
pub type BannedTokenStoreType = Arc<HashsetBannedTokenStore>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedTokenStoreType) -> Self {
        Self {
            user_store,
            banned_token_store,
        }
    }
}

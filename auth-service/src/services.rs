//! Pluggable store and client traits plus their backends.
//!
//! Each capability (user store, banned-token store, 2FA-code store, email client) is a trait with
//! interchangeable in-memory and networked implementations, selected by config at startup and
//! held in [`AppState`](crate::app_state::AppState) behind the generated `Arc<dyn _>` newtype
//! wrappers (`UserStoreType`, `EmailClientType`, …).

mod arc_dyn;

pub(crate) mod stores;
pub use stores::HashmapTwoFactorAuthCodeStore;
pub use stores::HashmapUserStore;
pub use stores::HashsetBannedTokenStore;
pub use stores::PostgresUserStore;
pub use stores::RedisBannedTokenStore;
pub use stores::RedisTwoFactorAuthCodeStore;

mod store_banned_tokens;
pub use store_banned_tokens::BannedTokenStore;
pub use store_banned_tokens::BannedTokenStoreError;
pub use store_banned_tokens::BannedTokenStoreResult;
pub use store_banned_tokens::BannedTokenStoreType;

mod store_tfa_codes;
pub use store_tfa_codes::TwoFactorAuthCodeStore;
pub use store_tfa_codes::TwoFactorAuthCodeStoreError;
pub use store_tfa_codes::TwoFactorAuthCodeStoreResult;
pub use store_tfa_codes::TwoFactorAuthCodeStoreType;

mod store_users;
pub use store_users::UserStore;
pub use store_users::UserStoreError;
pub use store_users::UserStoreResult;
pub use store_users::UserStoreType;
pub use store_users::warm_decoy_password_hash;

pub(crate) mod clients;
pub use clients::MockEmailClient;
pub use clients::PostmarkEmailClient;

mod client_email;
pub use client_email::EmailClient;
pub use client_email::EmailClientError;
pub use client_email::EmailClientResult;
pub use client_email::EmailClientType;

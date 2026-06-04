use crate::config::ConfigType;
use crate::domain::Secret;
use crate::services::BannedTokenStoreType;
use crate::services::EmailClientType;
use crate::services::TwoFactorAuthCodeStoreType;
use crate::services::UserStoreType;
use crate::utils::api_error::ApiError;
use crate::utils::api_error::ApiResult;

/// Shared application state injected into every handler.
///
/// Each field is an `Arc`-backed handle, so `AppState` is cheap to [`Clone`] per request — the
/// clone bumps reference counts rather than duplicating stores, clients, or config.
#[derive(Debug, Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_factor_auth_code_store: TwoFactorAuthCodeStoreType,
    pub email_client: EmailClientType,
    pub config: ConfigType,
}

impl AppState {
    /// Assemble the application state from its already-constructed dependencies.
    #[must_use]
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

    /// Clone out the configured JWT signing secret for token operations.
    ///
    /// The secret is loaded and validated at startup (see `JwtConfig::load_mandatory_arguments`),
    /// so a missing value here indicates a server-side configuration fault rather than bad input.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::UnexpectedError`] if the JWT secret was never loaded into the config.
    pub fn jwt_secret(&self) -> ApiResult<Secret> {
        self.config
            .inner()
            .jwt
            .jwt_secret
            .clone()
            .ok_or_else(|| ApiError::UnexpectedError(color_eyre::eyre::eyre!("JWT secret is not set.")))
    }
}

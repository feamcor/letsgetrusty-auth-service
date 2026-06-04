use crate::domain::Secret;
use rand::RngExt;

const TWO_FACTOR_AUTH_CODE_LENGTH: usize = 6;

/// A six-digit numeric two-factor authentication code.
///
/// [`Default`] mints a fresh random code; [`TwoFactorAuthCode::parse`] validates client input.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct TwoFactorAuthCode(Secret);

/// Error returned when a 2FA code is not exactly six ASCII digits.
#[derive(thiserror::Error, Debug)]
#[error("Invalid 2FA Code")]
pub struct TwoFactorAuthCodeError;

impl TwoFactorAuthCode {
    /// Parse a client-supplied 2FA code, requiring exactly six ASCII digits.
    ///
    /// # Errors
    ///
    /// Returns [`TwoFactorAuthCodeError`] if the code is the wrong length or contains non-digits.
    pub fn parse(code: &Secret) -> Result<Self, TwoFactorAuthCodeError> {
        let raw_code = code.expose();
        if raw_code.len() == TWO_FACTOR_AUTH_CODE_LENGTH && raw_code.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(code.to_owned()))
        } else {
            Err(TwoFactorAuthCodeError)
        }
    }

    #[must_use]
    pub fn as_secret(&self) -> &Secret {
        &self.0
    }
}

impl Default for TwoFactorAuthCode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code: u32 = rng.random_range(0..1_000_000);
        let code = format!("{code:06}");
        let code = code.into();
        Self(code)
    }
}

impl PartialEq for TwoFactorAuthCode {
    fn eq(&self, other: &Self) -> bool {
        self.as_secret() == other.as_secret()
    }
}

impl Eq for TwoFactorAuthCode {}

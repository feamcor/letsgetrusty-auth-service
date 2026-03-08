use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const TWO_FACTOR_AUTH_CODE_LENGTH: usize = 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TwoFactorAuthCode(String);

#[derive(thiserror::Error, Debug)]
#[error("Invalid 2FA Code: {0}")]
pub struct TwoFactorAuthCodeError(String);

impl TwoFactorAuthCode {
    pub fn parse(code: String) -> Result<Self, TwoFactorAuthCodeError> {
        if code.len() == TWO_FACTOR_AUTH_CODE_LENGTH && code.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(code))
        } else {
            Err(TwoFactorAuthCodeError(format!(
                "{code} is not a valid {TWO_FACTOR_AUTH_CODE_LENGTH}-digit 2FA code",
            )))
        }
    }
}

impl Display for TwoFactorAuthCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "2FA Code: {}", self.0)
    }
}

impl Default for TwoFactorAuthCode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code: u32 = rng.random_range(0..1_000_000);
        Self(format!("{code:06}"))
    }
}

impl AsRef<str> for TwoFactorAuthCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

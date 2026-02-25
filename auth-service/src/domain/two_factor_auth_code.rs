use rand::RngExt;

const TWO_FA_CODE_LENGTH: usize = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFactorAuthCode(String);

impl TwoFactorAuthCode {
    pub fn parse(code: String) -> Result<Self, String> {
        if code.len() == TWO_FA_CODE_LENGTH && code.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(code))
        } else {
            Err(format!("{} is not a valid 6-digit 2FA code", code))
        }
    }
}

impl Default for TwoFactorAuthCode {
    fn default() -> Self {
        let mut rng = rand::rng();
        let code: u32 = rng.random_range(0..1_000_000);
        Self(format!("{:06}", code))
    }
}

impl AsRef<str> for TwoFactorAuthCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

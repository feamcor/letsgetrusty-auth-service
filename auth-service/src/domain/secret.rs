use secrecy::ExposeSecret;
use secrecy::SecretString;
use subtle::ConstantTimeEq;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Secret(SecretString);

impl Secret {
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Self(secret.into())
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        // Use constant-time comparison so callers (Token, TwoFactorAuthCode, LoginAttemptId,
        // HashedPassword) can't leak the secret one byte at a time through response timing.
        self.expose().as_bytes().ct_eq(other.expose().as_bytes()).into()
    }
}

impl Eq for Secret {}

impl From<&str> for Secret {
    fn from(value: &str) -> Self {
        Self::from(String::from(value))
    }
}

impl From<String> for Secret {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl std::hash::Hash for Secret {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.expose().hash(state);
    }
}

/// Default `Serialize` deliberately redacts the inner value. Types that legitimately need to
/// emit the cleartext (e.g. `LoginAttemptId` in a JSON response body) must implement `Serialize`
/// themselves, or annotate the field with `#[serde(serialize_with = "Secret::expose_serializer")]`.
impl serde::Serialize for Secret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str("[REDACTED]")
    }
}

impl Secret {
    /// Opt-in serializer that emits the cleartext. Intended for `#[serde(serialize_with = ...)]`
    /// on individual fields whose plaintext must reach the wire.
    pub fn expose_serializer<S>(secret: &Secret, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(secret.expose())
    }
}

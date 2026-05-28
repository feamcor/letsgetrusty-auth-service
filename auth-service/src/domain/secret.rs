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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_secrets_compare_equal() {
        let a: Secret = "hello".into();
        let b: Secret = "hello".into();
        assert_eq!(a, b);
    }

    #[test]
    fn different_secrets_compare_unequal() {
        let a: Secret = "hello".into();
        let b: Secret = "world".into();
        assert_ne!(a, b);
    }

    #[test]
    fn different_length_secrets_compare_unequal() {
        // Constant-time comparison should still return false for inputs of different length.
        let a: Secret = "hello".into();
        let b: Secret = "helloworld".into();
        assert_ne!(a, b);
    }

    #[test]
    fn default_serialize_is_redacted() {
        let secret: Secret = "very-secret-value".into();
        let json = serde_json::to_string(&secret).unwrap();
        assert_eq!(json, r#""[REDACTED]""#);
        assert!(!json.contains("very-secret-value"));
    }

    #[test]
    fn expose_serializer_emits_cleartext() {
        #[derive(serde::Serialize)]
        struct Wrapper(#[serde(serialize_with = "Secret::expose_serializer")] Secret);
        let secret: Secret = "very-secret-value".into();
        let wrapper = Wrapper(secret);
        let json = serde_json::to_string(&wrapper).unwrap();
        assert_eq!(json, r#""very-secret-value""#);
    }

    #[test]
    fn debug_format_is_redacted() {
        // secrecy::SecretString's Debug already redacts; verify we haven't accidentally bypassed.
        let secret: Secret = "very-secret-value".into();
        let debug = format!("{secret:?}");
        assert!(!debug.contains("very-secret-value"), "Debug must not contain cleartext: {debug}");
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

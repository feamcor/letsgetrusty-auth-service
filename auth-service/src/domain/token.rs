use crate::domain::Secret;

/// An opaque bearer token (a JWT) carried as a redacted [`Secret`].
///
/// `Token` performs no validation itself — it is a typed wrapper that keeps raw token strings out
/// of logs and enables constant-time equality via [`Secret`]. Use `utils::auth` to mint/verify.
#[derive(Debug, Clone)]
pub struct Token(Secret);

impl Token {
    /// Wrap an existing token string.
    #[must_use]
    pub fn new(token: &Secret) -> Self {
        Self(token.to_owned())
    }

    /// Borrow the underlying redacted secret (e.g. to expose at the JWT boundary).
    #[must_use]
    pub fn as_secret(&self) -> &Secret {
        &self.0
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.as_secret() == other.as_secret()
    }
}

impl Eq for Token {}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_secret().hash(state);
    }
}

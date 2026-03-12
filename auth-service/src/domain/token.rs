use crate::domain::Secret;

#[derive(Debug, Clone)]
pub struct Token(Secret);

impl Token {
    #[must_use]
    pub fn new(token: &Secret) -> Self {
        Self(token.to_owned())
    }

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

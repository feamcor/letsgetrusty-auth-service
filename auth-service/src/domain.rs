//! Validated domain newtypes that own all input validation.
//!
//! Every type here is constructed through a fallible `parse`/`new` that enforces its invariants
//! once, so the rest of the codebase can trust the value thereafter. Sensitive values wrap
//! [`Secret`] for redaction and constant-time comparison.

mod email;
mod login_attempt_id;
mod password;
mod secret;
mod token;
mod two_factor_auth_code;
mod user;

pub use email::Email;
pub use email::EmailError;
pub use email::EmailResult;
pub use login_attempt_id::LoginAttemptId;
pub use login_attempt_id::LoginAttemptIdError;
pub use password::ARGON2_ITERATIONS;
pub use password::ARGON2_MEMORY_KIB;
pub use password::ARGON2_PARALLELISM;
pub use password::HashedPassword;
pub use password::MAX_PASSWORD_LENGTH;
pub use password::MIN_PASSWORD_LENGTH;
pub use password::PASSWORD_LENGTH_RANGE;
pub use password::PasswordError;
pub use password::PasswordResult;
pub use password::SAFE_PASSWORD_LENGTH_RANGE;
pub use password::compute_password_hash_sync;
pub use password::validate_password_strength;
pub use secret::Secret;
pub use token::Token;
pub use two_factor_auth_code::TwoFactorAuthCode;
pub use two_factor_auth_code::TwoFactorAuthCodeError;
pub use user::User;

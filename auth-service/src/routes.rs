//! Axum handlers for the auth API, one module per endpoint. Each returns
//! [`ApiResult`](crate::utils::api_error::ApiResult) so failures map to typed HTTP responses.

mod health;
pub use health::health;

mod login;
pub use login::LoginRequest;
pub use login::TwoFactorAuthResponse;
pub use login::login;

mod logout;
pub use logout::logout;

mod signup;
pub use signup::SignupRequest;
pub use signup::SignupResponse;
pub use signup::signup;

mod verify_2fa;
pub use verify_2fa::Verify2FARequest;
pub use verify_2fa::verify_2fa;

mod verify_token;
pub use verify_token::VerifyTokenRequest;
pub use verify_token::verify_token;

use lazy_static::lazy_static;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
}

fn set_token() -> String {
    dotenvy::dotenv().ok();
    let secret = std::env::var(env::JWT_SECRET)
        .expect(&format!("{} environment variable must be set", env::JWT_SECRET));
    if secret.is_empty() {
        panic!("{} environment variable must not be empty", env::JWT_SECRET);
    }
    secret
}

pub mod env {
    pub const JWT_SECRET: &str = "AUTH_SERVICE_JWT_SECRET";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const JWT_TTL_SECONDS: i64 = 10 * 60; // 10 minutes

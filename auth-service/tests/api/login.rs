use crate::helpers::TestApp;
use crate::helpers::TestAppAsyncContext;
use auth_service::domain::Email;
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::auth::JWT_COOKIE_NAME;
use fake::Fake;
use fake::faker::internet::en::DomainSuffix;
use fake::faker::internet::en::Password;
use fake::faker::internet::en::SafeEmail;
use mime::APPLICATION_JSON;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use serde_json::json;
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [json!({
        "email": SafeEmail().fake::<String>().as_str(),
        "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
    })];
    for request in &requests {
        let signup_request = json!({
            "email": request.get("email").unwrap().as_str(),
            "password": request.get("password").unwrap().as_str(),
            "requires2FA": false,
        });
        let signup_response = app.post_signup(&signup_request).await;
        assert_eq!(signup_response.status(), StatusCode::CREATED);
        let response = app.post_login(&request).await;
        assert_eq!(response.status(), StatusCode::OK);
        let jwt = response
            .cookies()
            .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
            .expect("No auth cookie found");
        assert!(!jwt.value().is_empty());
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [json!({
        "email": SafeEmail().fake::<String>().as_str(),
        "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
    })];
    for request in &requests {
        let signup_request = json!({
            "email": request.get("email").unwrap().as_str(),
            "password": request.get("password").unwrap().as_str(),
            "requires2FA": true,
        });
        let signup_response = app.post_signup(&signup_request).await;
        assert_eq!(signup_response.status(), StatusCode::CREATED);
        let response = app.post_login(&request).await;
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
        let body = response
            .json::<TwoFactorAuthResponse>()
            .await
            .expect("Failed to deserialize TwoFactorAuthResponse");
        let email = request.get("email").unwrap();
        let email = email.as_str().unwrap().into();
        let email = Email::parse(&email).unwrap();
        let (stored_login_attempt_id, _) = app
            .two_factor_auth_code_store
            .inner()
            .get_code(&email)
            .await
            .expect("Login attempt ID not found in store");
        assert_eq!(stored_login_attempt_id, body.login_attempt_id);
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_400_if_invalid_input(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": Password(1..7).fake::<String>().as_str()
        }),
        json!({
            "email": DomainSuffix().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
        json!({
            "email": DomainSuffix().fake::<String>().as_str(),
            "password": Password(1..7).fake::<String>().as_str()
        }),
    ];
    for request in &requests {
        let response = app.post_login(request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [json!({
        "email": SafeEmail().fake::<String>().as_str(),
        "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
    })];
    for request in &requests {
        let signup_request = json!({
            "email": request.get("email").unwrap().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
            "requires2FA": false,
        });
        let signup_response = app.post_signup(&signup_request).await;
        assert_eq!(signup_response.status(), StatusCode::CREATED);
        let response = app.post_login(&request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_422_if_unprocessable_content(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [
        json!({"email": SafeEmail().fake::<String>().as_str()}),
        json!({"password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()}),
    ];
    for request in &requests {
        let response = app.post_login(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Input: {request:?}"
        );
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_500_if_unexpected_error(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests: [Value; 0] = [];
    for request in &requests {
        let response = app.post_login(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Input: {request:?}"
        );
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
    }
}

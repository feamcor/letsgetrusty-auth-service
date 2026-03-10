use crate::helpers::TestApp;
use crate::helpers::TestAppAsyncContext;
use auth_service::domain::Email;
use auth_service::domain::LoginAttemptId;
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::domain::TwoFactorAuthCode;
use auth_service::routes::TwoFactorAuthResponse;
use fake::Fake;
use fake::faker::internet::en::DomainSuffix;
use fake::faker::internet::en::SafeEmail;
use mime::APPLICATION_JSON;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn verify_2fa_successful(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let email = SafeEmail().fake::<String>();
    let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
    let signup_request = json!({
        "email": &email,
        "password": &password,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_request).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let login_request = json!({
        "email": &email,
        "password": &password,
    });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap()
        .login_attempt_id;
    let request = json!({
        "email": email,
        "loginAttemptId": attempt_id,
        "2FACode": TwoFactorAuthCode::default(),
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_400_if_invalid_input(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [
        json!({
            "email": DomainSuffix().fake::<String>().as_str(),
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": TwoFactorAuthCode::default(),
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "loginAttemptId": "invalid",
            "2FACode": TwoFactorAuthCode::default(),
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "loginAttemptId": LoginAttemptId::default(),
            "2FACode": "invalid",
        }),
    ];
    for request in &requests {
        let response = app.post_verify_2fa(&request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let email = SafeEmail().fake::<String>();
    let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
    let signup_request = json!({
        "email": &email,
        "password": &password,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_request).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let login_request = json!({
        "email": &email,
        "password": &password,
    });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap()
        .login_attempt_id;
    let request = json!({
        "email": email,
        "loginAttemptId": attempt_id,
        "2FACode": TwoFactorAuthCode::default(),
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_old_attempt_id(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let email = SafeEmail().fake::<String>();
    let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
    let signup_request = json!({
        "email": &email,
        "password": &password,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_request).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let login_request = json!({
        "email": &email,
        "password": &password,
    });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap()
        .login_attempt_id;
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let store = &app.two_factor_auth_code_store;
    let (_, auth_code) = store
        .inner()
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    let request = json!({
        "email": email,
        "loginAttemptId": attempt_id, // this is the attempt id of the first login
        "2FACode": auth_code, // this is the auth code of the second login
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_old_auth_code(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let email = SafeEmail().fake::<String>();
    let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
    let signup_request = json!({
        "email": &email,
        "password": &password,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_request).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let login_request = json!({
        "email": &email,
        "password": &password,
    });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let store = &app.two_factor_auth_code_store;
    let (_, auth_code) = store
        .inner()
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let attempt_id = login_response
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap()
        .login_attempt_id;
    let request = json!({
        "email": email,
        "loginAttemptId": attempt_id, // this is the attempt id of the second login
        "2FACode": auth_code, // this is the auth code of the first login
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_same_code_twice(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let email = SafeEmail().fake::<String>();
    let password = SAFE_PASSWORD_LENGTH_RANGE.fake::<String>();
    let signup_request = json!({
        "email": &email,
        "password": &password,
        "requires2FA": true,
    });
    let signup_response = app.post_signup(&signup_request).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);
    let login_request = json!({
        "email": &email,
        "password": &password,
    });
    let login_response = app.post_login(&login_request).await;
    assert_eq!(login_response.status(), StatusCode::PARTIAL_CONTENT);
    let store = &app.two_factor_auth_code_store;
    let (attempt_id, auth_code) = store
        .inner()
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    let request = json!({
        "email": email,
        "loginAttemptId": attempt_id,
        "2FACode": auth_code,
    });
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = app.post_verify_2fa(&request).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_422_if_malformed_input(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests = [
        json!(null),
        json!(true),
        json!("string"),
        json!(0),
        json!([]),
        json!({}),
        json!({"key": "value"}),
        json!({"email": null}),
        json!({"email": true}),
        json!({"email": 0}),
        json!({"email": []}),
        json!({"email": {}}),
        json!({"email": "string", "loginAttemptId": "string"}),
        json!({"loginAttemptId": "string", "2FACode": "string"}),
        json!({"2FACode": "string", "email": "string"}),
    ];
    for request in &requests {
        let response = app.post_verify_2fa(&request).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

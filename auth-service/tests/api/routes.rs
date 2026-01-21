use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response = app.get_root().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn signup_user_created_successfully() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string", "requires2FA": true});
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::CREATED);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn signup_invalid_input() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string", "requires2FA": true});
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn signup_email_already_exists() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string", "requires2FA": true});
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::CONFLICT);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn signup_unprocessable_content() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string", "requires2FA": true});
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn signup_unexpected_error() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string", "requires2FA": true});
    let response = app.post_signup(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn login_successful() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK);
    //let jwt = jwt_cookie(&response);
    //assert_jwt(jwt);
}

#[tokio::test]
async fn login_requires_2fa() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn login_invalid_input() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn login_authentication_failed() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "wrong_password"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn login_unprocessable_content() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn login_unexpected_error() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "password": "string"});
    let response = app.post_login(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    //assert_eq!(response.status(), StatusCode::OK);assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn verify_2fa_successful() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK);
    //let jwt = jwt_cookie(&response);
    //assert_jwt(jwt);
}

#[tokio::test]
async fn verify_2fa_invalid_input() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn verify_2fa_authentication_failed() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    //assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn verify_2fa_unprocessable_content() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    //assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn verify_2fa_unexpected_error() {
    let app = TestApp::new().await;
    let body = json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn logout_successful() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK);
    // let jwt = jwt_cookie(&response);
    // let jwt = assert_jwt(jwt);
    // assert!(jwt.contains("Expires=Thu, 01 Jan 1970 00:00:00 GMT;"), "JWT must have Expires set as epoch");
}

#[tokio::test]
async fn logout_invalid_input() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn logout_jwt_is_not_valid() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn logout_unexpected_error() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn verify_token_is_valid() {
    let app = TestApp::new().await;
    let body = json!({"token":"string"});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_token_jwt_is_not_valid() {
    let app = TestApp::new().await;
    let body = json!({"token":"string"});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn verify_token_unprocessable_content() {
    let app = TestApp::new().await;
    let body = json!({"token":"string"});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn verify_token_unexpected_error() {
    let app = TestApp::new().await;
    let body = json!({"token":"string"});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

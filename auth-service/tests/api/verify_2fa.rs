use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;

#[allow(unused_imports)]
use mime::APPLICATION_JSON;

#[allow(unused_imports)]
use reqwest::header::CONTENT_TYPE;

#[tokio::test]
async fn verify_2fa_successful() {
    let app = TestApp::new().await;
    let body =
        json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK);
    //let jwt = jwt_cookie(&response);
    //assert_jwt(jwt);
}

#[tokio::test]
async fn verify_2fa_invalid_input() {
    let app = TestApp::new().await;
    let body =
        json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
}

#[tokio::test]
async fn verify_2fa_authentication_failed() {
    let app = TestApp::new().await;
    let body =
        json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
}

#[tokio::test]
async fn verify_2fa_unprocessable_content() {
    let app = TestApp::new().await;
    let body =
        json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn verify_2fa_unexpected_error() {
    let app = TestApp::new().await;
    let body =
        json!({"email": "user@example.com", "loginAttemptId": "string", "2FACode": "string"});
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
}

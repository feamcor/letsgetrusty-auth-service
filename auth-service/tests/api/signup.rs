use reqwest::StatusCode;
use serde_json::json;
use crate::helpers::TestApp;

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

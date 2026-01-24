use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;

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

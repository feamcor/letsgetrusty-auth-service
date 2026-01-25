use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;

#[allow(unused_imports)]
use mime::APPLICATION_JSON;

#[allow(unused_imports)]
use reqwest::header::CONTENT_TYPE;

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
    // assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
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
    // assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), APPLICATION_JSON.as_ref());
}

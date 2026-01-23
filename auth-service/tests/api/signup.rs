use crate::helpers::{random_email, TestApp};
use auth_service::routes::SignupResponse;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let expected = SignupResponse::Message("User created successfully!".to_string());
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": "alice@example.com",
            "password": "StrongPassword123!",
            "requires2FA": false,
        }),
        json!({
            "email": "bob@example.com",
            "password": "StrongPassword456!",
            "requires2FA": true,
        }),
    ];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
        assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": "example.com",
            "password": "StrongPassword123!",
            "requires2FA": false,
        }),
        json!({
            "email": "alice@example.com",
            "password": "Weak!",
            "requires2FA": false,
        }),
    ];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Input: {:?}", request);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    }
}

#[tokio::test]
async fn should_return_409_if_user_already_exists() {
    let app = TestApp::new().await;
    let request = json!({
        "email": "alice@example.com",
        "password": "StrongPassword123!",
        "requires2FA": false,
    });
    let response = app.post_signup(&request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");

    let response = app.post_signup(&request).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn should_return_422_if_unprocessable_content() {
    let app = TestApp::new().await;
    let _email = random_email();
    let requests = [
        json!({"password": "password123", "requires2FA": true})];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Input: {:?}",
            request
        );
    }
}

#[tokio::test]
async fn should_return_500_if_unexpected_error() {
    let app = TestApp::new().await;
    let requests: [Value; 0] = [];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR, "Input: {:?}", request);
        assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
    }
}

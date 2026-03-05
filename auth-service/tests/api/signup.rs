use crate::helpers::TestApp;
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::routes::SignupResponse;
use fake::Fake;
use fake::faker::internet::en::{DomainSuffix, Password, SafeEmail};
use mime::APPLICATION_JSON;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::{Value, json};

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let expected = SignupResponse { message: "User created successfully".to_string() };
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
            "requires2FA": false,
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
            "requires2FA": true,
        }),
    ];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
        assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected);
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": DomainSuffix().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
            "requires2FA": false,
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": Password(1..7).fake::<String>().as_str(),
            "requires2FA": false,
        }),
    ];
    for request in requests.iter() {
        let response = app.post_signup(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Input: {:?}",
            request
        );
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

#[tokio::test]
async fn should_return_409_if_user_already_exists() {
    let app = TestApp::new().await;
    let request = json!({
        "email": SafeEmail().fake::<String>().as_str(),
        "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
        "requires2FA": false,
    });
    let response = app.post_signup(&request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );

    let response = app.post_signup(&request).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[tokio::test]
async fn should_return_422_if_unprocessable_content() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
        json!({
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str(),
            "requires2FA": false
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "requires2FA": false
        }),
        json!({
            "email": SafeEmail().fake::<String>().as_str()
        }),
        json!({
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
        json!({
            "requires2FA": false
        }),
    ];
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
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Input: {:?}",
            request
        );
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

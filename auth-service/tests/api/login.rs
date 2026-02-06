use crate::helpers::TestApp;
use fake::faker::internet::en::{DomainSuffix, Password, SafeEmail};
use fake::Fake;
use mime::APPLICATION_JSON;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::utils::constants::JWT_COOKIE_NAME;

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
    ];
    for request in requests.iter() {
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

#[tokio::test]
async fn should_return_206_if_login_requires_2fa() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
    ];
    for request in requests.iter() {
        let signup_request = json!({
            "email": request.get("email").unwrap().as_str(),
            "password": request.get("password").unwrap().as_str(),
            "requires2FA": true,
        });
        let signup_response = app.post_signup(&signup_request).await;
        assert_eq!(signup_response.status(), StatusCode::CREATED);
        // TODO: 2FA should be verified here
        let response = app.post_login(&request).await;
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
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
    for request in requests.iter() {
        let response = app.post_login(request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let requests = [
        json!({
            "email": SafeEmail().fake::<String>().as_str(),
            "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
        }),
    ];
    for request in requests.iter() {
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

#[tokio::test]
async fn should_return_422_if_unprocessable_content() {
    let app = TestApp::new().await;
    let requests = [
        json!({"email": SafeEmail().fake::<String>().as_str()}),
        json!({"password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()}),
    ];
    for request in requests.iter() {
        let response = app.post_login(&request).await;
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
        let response = app.post_login(&request).await;
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

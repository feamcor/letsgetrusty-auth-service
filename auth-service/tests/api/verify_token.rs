use crate::helpers::TestApp;
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use fake::faker::internet::en::SafeEmail;
use fake::Fake;
use mime::APPLICATION_JSON;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn should_return_200_if_valid_token() {
    let app = TestApp::new().await;
    let login_request = json!({
        "email": SafeEmail().fake::<String>().as_str(),
        "password": SAFE_PASSWORD_LENGTH_RANGE.fake::<String>().as_str()
    });
    let signup_request = json!({
        "email": login_request.get("email").unwrap().as_str(),
        "password": login_request.get("password").unwrap().as_str(),
        "requires2FA": false,
    });
    let response = app.post_signup(&signup_request).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = app.post_login(&login_request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let jwt = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    let request = json!({"token": jwt.value()});
    let response = app.post_verify_token(&request).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;
    let body = json!({"token":"string"});
    let response = app.post_verify_token(&body).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let requests = [
        json!(null),
        json!(true),
        json!("string"),
        json!(0),
        json!([]),
        json!({}),
        json!({"key": "value"}),
        json!({"token": null}),
        json!({"token": true}),
        json!({"token": 0}),
        json!({"token": []}),
        json!({"token": {}}),
        json!({"token": "string", "key": "value"}),
    ];
    for request in requests.iter() {
        let response = app.post_verify_token(&request).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

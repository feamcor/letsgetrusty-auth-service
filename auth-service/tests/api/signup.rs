use crate::helpers::{TestApp, TestAppAsyncContext};
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::routes::SignupResponse;
use fake::faker::internet::en::{DomainSuffix, Password, SafeEmail};
use fake::Fake;
use mime::APPLICATION_JSON;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use serde_json::{json, Value};
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_201_if_valid_input(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let expected = SignupResponse {
        message: "User created successfully".to_string(),
    };
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
    for request in &requests {
        let response = app.post_signup(&request).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
        assert_eq!(response.json::<SignupResponse>().await.unwrap(), expected);
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_400_if_invalid_input(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
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
    for request in &requests {
        let response = app.post_signup(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Input: {request:?}"
        );
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_409_if_user_already_exists(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
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

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_422_if_unprocessable_content(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
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
    for request in &requests {
        let response = app.post_signup(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Input: {request:?}"
        );
    }
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_500_if_unexpected_error(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let requests: [Value; 0] = [];
    for request in &requests {
        let response = app.post_signup(&request).await;
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Input: {request:?}"
        );
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            APPLICATION_JSON.as_ref()
        );
    }
}

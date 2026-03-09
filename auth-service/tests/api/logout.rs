use crate::helpers::{TestApp, TestAppAsyncContext};
use auth_service::domain::SAFE_PASSWORD_LENGTH_RANGE;
use auth_service::utils::auth::JWT_COOKIE_NAME;
use fake::faker::internet::en::SafeEmail;
use fake::Fake;
use mime::APPLICATION_JSON;
use reqwest::header::CONTENT_TYPE;
use reqwest::{StatusCode, Url};
use serde_json::json;
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_200_if_valid_jwt(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
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
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK);
    let is_banned = app
        .banned_token_store
        .inner()
        .is_token_banned(jwt.value())
        .await;
    assert!(is_banned.unwrap());
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
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
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK);
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_401_if_invalid_token(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    app.cookie_jar.add_cookie_str(
        &format!(
            "{JWT_COOKIE_NAME}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/"
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        APPLICATION_JSON.as_ref()
    );
}

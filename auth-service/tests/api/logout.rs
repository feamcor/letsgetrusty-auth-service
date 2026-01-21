use reqwest::StatusCode;
use crate::helpers::TestApp;

#[tokio::test]
async fn logout_successful() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK);
    // let jwt = jwt_cookie(&response);
    // let jwt = assert_jwt(jwt);
    // assert!(jwt.contains("Expires=Thu, 01 Jan 1970 00:00:00 GMT;"), "JWT must have Expires set as epoch");
}

#[tokio::test]
async fn logout_invalid_input() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn logout_jwt_is_not_valid() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

#[tokio::test]
async fn logout_unexpected_error() {
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::OK); // TODO: dummy assertion for task 4
    // assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    // assert_eq!(response.headers().get("content-type").unwrap(), "application/json");
}

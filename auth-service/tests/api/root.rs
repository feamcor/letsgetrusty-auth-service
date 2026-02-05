use crate::helpers::TestApp;
use mime::TEXT_HTML;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response = app.get_root().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        TEXT_HTML.as_ref()
    );
}

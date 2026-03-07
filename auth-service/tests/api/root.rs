use crate::helpers::{TestApp, TestAppAsyncContext};
use mime::TEXT_HTML;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_200_if_returns_auth_ui(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let response = app.get_root().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        TEXT_HTML.as_ref()
    );
}

use crate::helpers::TestApp;
use crate::helpers::TestAppAsyncContext;
use mime::TEXT_HTML;
use reqwest::StatusCode;
use reqwest::header::CONTENT_TYPE;
use test_context::test_context;

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn should_return_200_if_returns_auth_ui(ctx: &mut TestAppAsyncContext) {
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let response = app.get_root().await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), TEXT_HTML.as_ref());
}

#[test_context(TestAppAsyncContext)]
#[tokio::test]
async fn responses_carry_defense_in_depth_security_headers(ctx: &mut TestAppAsyncContext) {
    // Defense-in-depth: confirm every response (root or API) carries the static security headers
    // we configure. If a future tower-http layer reorder strips them, this test catches it.
    let app = TestApp::new(ctx.db_name.as_str()).await;
    ctx.db_url = app.db_url.clone();
    let response = app.get_root().await;
    let headers = response.headers();
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(
        headers.get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );
}

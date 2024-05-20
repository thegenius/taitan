use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;

use taitan::application::Application;
use taitan_test::checked_oneshot;
use taitan_test::ResponseBuilder;
use taitan_test::ValidationError;

#[tokio::test]
async fn default_dev() {
    let app = Application::default_dev();
    let router = app.get_router();

    let expect = ResponseBuilder::ok_empty();
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    checked_oneshot(router, req, expect).await;

    let expect = ResponseBuilder::ok_empty();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    checked_oneshot(router, req, expect).await;

    let expect = ResponseBuilder::ok_empty();
    let req = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    checked_oneshot(router, req, expect).await;
}

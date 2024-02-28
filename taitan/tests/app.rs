mod common;
mod response;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;

use common::checked_oneshot;
use common::ValidationError;
use response::ResponseBuilder;
use taitan::application::Application;

#[tokio::test]
async fn default_dev() {
    let app = Application::default_dev();
    let router = app.get_router();

    let expect = ResponseBuilder::empty_ok();
    let req = Request::builder().uri("/").body(Body::empty()).unwrap();
    checked_oneshot(router, req, expect).await;

    let expect = ResponseBuilder::empty_ok();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    checked_oneshot(router, req, expect).await;

    let expect = ResponseBuilder::empty_ok();
    let req = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();
    checked_oneshot(router, req, expect).await;
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;

pub struct ResponseBuilder {}

impl ResponseBuilder {
    pub fn empty_ok() -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()
    }
}

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use serde::Serialize;
use std::borrow::Cow;
use taitan::response::ApiResponse;

pub struct ResponseBuilder {}

impl ResponseBuilder {
    pub fn ok_empty() -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()
    }

    pub fn ok_json<T: Serialize>(data: &T) -> Response {
        let bytes = serde_json::to_string(data).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    }

    pub fn api_success<T: Serialize>(data: &T) -> Response {
        let response = taitan::response::ApiResponse::success(data);
        let bytes = serde_json::to_string(&response).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    }

    pub fn api_success_opt<T: Serialize>(data: Option<T>) -> Response {
        let response = taitan::response::ApiResponse::success_opt(data);
        let bytes = serde_json::to_string(&response).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    }

    pub fn api_success_array<T: Serialize + Clone>(data: impl AsRef<[T]>) -> Response {
        let response: ApiResponse<'_, T> = taitan::response::ApiResponse::<T>::success_array(data);
        let bytes = serde_json::to_string(&response).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    }

    pub fn api_failure<'a>(err_code: u16, err_msg: impl Into<Cow<'a, str>>) -> Response {
        let response: ApiResponse<'_, ()> =
            taitan::response::ApiResponse::failure(err_code, err_msg);
        let bytes = serde_json::to_string(&response).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(bytes))
            .unwrap()
    }
}

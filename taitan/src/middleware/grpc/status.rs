use axum::response::{IntoResponse, Response};
use std::ops::{Deref, DerefMut};
// use axum_core::response::into_response::IntoResponse;
use tonic::Status;
// use tonic::metadata::GRPC_CONTENT_TYPE;
use bytes::Bytes;
use http::HeaderValue;
use http_body_util::{combinators::BoxBody, combinators::UnsyncBoxBody, BodyExt, Empty, Full};
use tonic::codegen::empty_body;
use std::borrow::Cow;

pub const GRPC_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/grpc");

#[derive(Debug)]
pub struct GrpcStatus(pub Status);

impl From<tonic::Status> for GrpcStatus {
    fn from(s: tonic::Status) -> Self {
        Self(s)
    }
}

impl Deref for GrpcStatus {
    type Target = tonic::Status;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GrpcStatus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

type ResponseBody = UnsyncBoxBody<Bytes, Status>;

impl IntoResponse for GrpcStatus {
    fn into_response(self) -> Response {
        // let status = self.0;
        // let msg: &str = self.message();
        // let msg = self.0.message();
        let msg = "grpc error";
        let body = axum::body::Body::from(msg);
        let mut response = http::Response::new(body);

        response
            .headers_mut()
            .insert(http::header::CONTENT_TYPE, GRPC_CONTENT_TYPE);

        response
    }
}

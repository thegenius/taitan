use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiSuccess<T> {
    success: bool,
    data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiSuccessOpt<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiSuccessArray<T> {
    success: bool,
    data: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiFailure<'a> {
    success: bool,
    err_code: u16,
    err_msg: Cow<'a, str>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ApiResponse<'a, T: Serialize> {
    Success(ApiSuccess<T>),
    SuccessOpt(ApiSuccessOpt<T>),
    SuccessArray(ApiSuccessArray<T>),
    Failure(ApiFailure<'a>),
}

impl<'a, T: Serialize> ApiResponse<'a, T> {
    pub fn success(val: T) -> ApiResponse<'a, T> {
        Self::Success(ApiSuccess {
            success: true,
            data: val,
        })
    }

    pub fn success_opt(val: Option<T>) -> ApiResponse<'a, T> {
        Self::SuccessOpt(ApiSuccessOpt {
            success: true,
            data: val,
        })
    }
    pub fn success_array<E: Clone + Serialize>(val: impl AsRef<[E]>) -> ApiResponse<'a, E> {
        ApiResponse::SuccessArray(ApiSuccessArray {
            success: true,
            data: val.as_ref().to_vec(),
        })
    }
    pub fn failure(err_code: u16, err_msg: impl Into<Cow<'a, str>>) -> ApiResponse<'a, T> {
        Self::Failure(ApiFailure {
            success: false,
            err_code,
            err_msg: err_msg.into(),
        })
    }

    fn to_json(&self) -> ApiResponseSerialized {
        let ser_err_msg = r#"{ "success": false, "errCode": "SerializeError"  "errMsg": "ApiResponse serialize error" }"#;
        match self {
            Self::Failure(data) => match serde_json::to_string(&data) {
                Ok(value) => ApiResponseSerialized::Failure(value.into()),
                Err(_) => ApiResponseSerialized::Failure(ser_err_msg.into()),
            },
            Self::Success(data) => match serde_json::to_string(&data) {
                Ok(value) => ApiResponseSerialized::Success(value.into()),
                Err(_) => ApiResponseSerialized::Failure(ser_err_msg.into()),
            },
            Self::SuccessOpt(data) => match serde_json::to_string(&data) {
                Ok(value) => ApiResponseSerialized::Success(value.into()),
                Err(_) => ApiResponseSerialized::Failure(ser_err_msg.into()),
            },
            Self::SuccessArray(data) => match serde_json::to_string(&data) {
                Ok(value) => ApiResponseSerialized::Success(value.into()),
                Err(_) => ApiResponseSerialized::Failure(ser_err_msg.into()),
            },
        }
    }
}

pub enum ApiResponseSerialized<'a> {
    Success(Cow<'a, str>),
    Failure(Cow<'a, str>),
}

impl<'a> From<ApiResponseSerialized<'a>> for Cow<'a, str> {
    fn from(value: ApiResponseSerialized<'a>) -> Self {
        match value {
            ApiResponseSerialized::Success(val) => val,
            ApiResponseSerialized::Failure(val) => val,
        }
    }
}

#[inline(always)]
pub fn build_response<'a>(statuc_code: StatusCode, msg: impl Into<Cow<'a, str>>) -> Response {
    let content: Cow<'a, str> = msg.into();
    Response::builder()
        .status(statuc_code)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(content.into_owned()))
        .unwrap()
}

impl<'a, T: Serialize> IntoResponse for ApiResponse<'a, T> {
    fn into_response(self) -> Response {
        let serilized = self.to_json();
        match serilized {
            ApiResponseSerialized::Success(val) => build_response(StatusCode::OK, val),
            ApiResponseSerialized::Failure(val) => {
                build_response(StatusCode::INTERNAL_SERVER_ERROR, val)
            }
        }
    }
}

impl<'a, T> From<T> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: T) -> Self {
        ApiResponse::<T>::Success(ApiSuccess {
            success: true,
            data: origin,
        })
    }
}

impl<'a, T> From<&T> for ApiResponse<'a, T>
where
    T: serde::Serialize + Clone,
{
    fn from(origin: &T) -> Self {
        ApiResponse::<T>::Success(ApiSuccess {
            success: true,
            data: origin.clone(),
        })
    }
}

impl<'a, T> From<Option<T>> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: Option<T>) -> Self {
        ApiResponse::<T>::SuccessOpt(ApiSuccessOpt {
            success: true,
            data: origin,
        })
    }
}

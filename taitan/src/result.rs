use crate::error::Error;
use crate::response::{ApiFailure, ApiResponse, ApiSuccess, ApiSuccessOpt};
pub type Result<T> = std::result::Result<T, Error>;
use axum::response::Json;

/* can not implement IntoResponse for Result
impl<T> IntoResponse for Result<T> {
    fn into_response(self) -> Response {
        match self {
            Ok(val) => ApiResponse::success(self).into(),
            Err(err) => err.into_response(),
        }
    }
}
*/
impl<'a, T> From<Result<T>> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: Result<T>) -> Self {
        match origin {
            Ok(data) => ApiResponse::success(data),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<Json<T>>> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: Result<Json<T>>) -> Self {
        match origin {
            Ok(data) => ApiResponse::success(data.0),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<Option<T>>> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: Result<Option<T>>) -> Self {
        match origin {
            Ok(data) => ApiResponse::<T>::success_opt(data),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<Json<Option<T>>>> for ApiResponse<'a, T>
where
    T: serde::Serialize,
{
    fn from(origin: Result<Json<Option<T>>>) -> Self {
        match origin {
            Ok(data) => ApiResponse::<T>::success_opt(data.0),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<Vec<T>>> for ApiResponse<'a, T>
where
    T: serde::Serialize + Clone,
{
    fn from(origin: Result<Vec<T>>) -> Self {
        match origin {
            Ok(data) => ApiResponse::<T>::success_array(data),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<Json<Vec<T>>>> for ApiResponse<'a, T>
where
    T: serde::Serialize + Clone,
{
    fn from(origin: Result<Json<Vec<T>>>) -> Self {
        match origin {
            Ok(data) => ApiResponse::<T>::success_array(data.0),
            Err(err) => ApiResponse::from(err),
        }
    }
}

impl<'a, T> From<Result<&[T]>> for ApiResponse<'a, T>
where
    T: serde::Serialize + Clone,
{
    fn from(origin: Result<&[T]>) -> Self {
        match origin {
            Ok(data) => ApiResponse::<T>::success_array(data),
            Err(err) => ApiResponse::from(err),
        }
    }
}

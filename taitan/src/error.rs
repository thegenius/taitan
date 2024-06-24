use crate::response::build_response;
use crate::response::ApiResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use chrono;
use jsonwebtoken;
use serde::Serialize;
use serde_json;
use std::borrow::Cow;
use thiserror;

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Success,
    LogicError,
    AxumError,
    AxumMultipartError,
    SerdeError,
    ChronoTimeParseError,
    DatabaseError,
    JwtError,
    FileError,
    ReqwestError,
    FromUtf8Error,
    UuidError,
}

impl ErrorCode {
    #[inline(always)]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::LogicError => "LogicError",
            Self::AxumError => "AxumError",
            Self::FromUtf8Error => "FromUtf8Error",
            Self::AxumMultipartError => "AxumMultipartError",
            Self::SerdeError => "SerdeError",
            Self::ChronoTimeParseError => "ChronoTimeParseError",
            Self::DatabaseError => "DatabaseError",
            Self::JwtError => "JwtError",
            Self::FileError => "FileError",
            Self::ReqwestError => "ReqwestError",
            Self::UuidError => "UuidError",
        }
    }

    #[inline(always)]
    pub fn value(&self) -> u16 {
        *self as u16
    }
}

#[allow(unused)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("logic error: {0}")]
    LogicError(String),

    #[error("axum error: {0:?}")]
    AxumError(#[from] axum::Error),

    #[error("axum multipart error: {0:?}")]
    AxumMultipartError(#[from] axum::extract::multipart::MultipartError),

    #[error("serde error ({0:?}")]
    SerdeError(#[from] serde_json::Error),

    #[error("chrono error ({0:?}")]
    ChronoTimeParseError(#[from] chrono::ParseError),

    #[error("database error ({0:?}")]
    DatabaseError(#[from] luna_orm::prelude::LunaOrmError),

    #[error("jwt error ({0:?}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("file error: {0:?}")]
    FileError(#[from] std::io::Error),

    #[error("file error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("from utf8 error: {0:?}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("uuid error: {0:?}")]
    UuidError(#[from] uuid::Error),
    // #[error("temp file error: {0:?}")]
    // TempFileError(#[from] tempfile::)
}

impl Error {
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::LogicError(_) => ErrorCode::LogicError,
            Error::AxumError(_) => ErrorCode::AxumError,
            Error::AxumMultipartError(_) => ErrorCode::AxumMultipartError,
            Error::SerdeError(_) => ErrorCode::SerdeError,
            Error::ChronoTimeParseError(_) => ErrorCode::ChronoTimeParseError,
            Error::DatabaseError(_) => ErrorCode::DatabaseError,
            Error::JwtError(_) => ErrorCode::JwtError,
            Error::FileError(_) => ErrorCode::FileError,
            Error::ReqwestError(_) => ErrorCode::ReqwestError,
            Error::FromUtf8Error(_) => ErrorCode::FromUtf8Error,
            Error::UuidError(_) => ErrorCode::UuidError,
        }
    }
    pub fn logic_error<'a>(msg: impl Into<Cow<'a, str>>) -> Self {
        let message: Cow<'a, str> = msg.into();
        Self::LogicError(message.to_string())
    }
}
impl<'a, T: Serialize> From<Error> for ApiResponse<'a, T> {
    fn from(err: Error) -> Self {
        let err_str = err.to_string();
        let api_failure: ApiResponse<'a, T> = ApiResponse::failure(err.code().value(), err_str);
        api_failure
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        ApiResponse::<()>::from(self).into_response()
    }
}

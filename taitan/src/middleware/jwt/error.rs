use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("invliad token")]
    InvalidToken,

    #[error("invliad token")]
    WrongCredentials,

    #[error("invliad token")]
    TokenCreation,

    #[error("invliad token")]
    MissingCredentials,

    #[error("json web token parse error")]
    JsonWebTokenError(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
            AuthError::JsonWebTokenError(err) => (StatusCode::UNAUTHORIZED, "jwt parse error"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

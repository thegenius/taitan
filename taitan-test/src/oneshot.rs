use axum::{
    body::Body,
    http::{self, Request, StatusCode},
    response::Response,
    Router,
};

use axum::body::Bytes;
use http_body_util::BodyExt; // for `collect`
use serde_json::{json, Value};
use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready`

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ValidationError {
    #[error("oneshot fail: {0}")]
    OneshotFail(String),
    #[error("json not match: {0} != {1}")]
    JsonNotMatch(String, String),
    #[error("validation error: body not readable {0}")]
    BodyReadFail(String),
    #[error("status code not match: return {0}, but expect {1}")]
    StatusCodeNotMatch(StatusCode, StatusCode),
    #[error("validation error: invalid json")]
    InvalidJson,
}

async fn read_buffer(body: Body) -> Result<Bytes, ValidationError> {
    match body.collect().await {
        Ok(data) => Ok(data.to_bytes()),
        Err(err) => Err(ValidationError::BodyReadFail(err.to_string())),
    }
}

async fn get_response(
    router: &Router,
    req: axum::extract::Request,
) -> Result<Response, ValidationError> {
    let response = router
        .clone()
        .oneshot(req)
        .await
        .map_err(|err| ValidationError::OneshotFail(err.to_string()))?;
    return Ok(response);
}

async fn validata_status_code(
    origin_response: &Response,
    expect_response: &Response,
) -> Result<(), ValidationError> {
    let status_code = origin_response.status();
    let expect_status_code = expect_response.status();
    if status_code != expect_status_code {
        return Err(ValidationError::StatusCodeNotMatch(
            status_code,
            expect_status_code,
        ));
    }
    return Ok(());
}

async fn validate_json_body(
    origin_response: Response,
    expect_response: Response,
) -> Result<(), ValidationError> {
    let origin_bytes = read_buffer(origin_response.into_body()).await?;
    let expect_bytes = read_buffer(expect_response.into_body()).await?;
    if origin_bytes.is_empty() && expect_bytes.is_empty() {
        return Ok(());
    }

    if origin_bytes.is_empty() != expect_bytes.is_empty() {
        return Err(ValidationError::JsonNotMatch(
            String::from_utf8(origin_bytes.to_vec()).unwrap_or("@invalid utf8 slice".to_string()),
            String::from_utf8(expect_bytes.to_vec()).unwrap_or("@invalid utf8 slice".to_string()),
        ));
    }

    let json: serde_json::Value =
        serde_json::from_slice(&origin_bytes).map_err(|err| ValidationError::InvalidJson)?;

    let expect_json: serde_json::Value =
        serde_json::from_slice(&expect_bytes).map_err(|err| ValidationError::InvalidJson)?;

    if json != expect_json {
        return Err(ValidationError::JsonNotMatch(
            json.to_string(),
            expect_json.to_string(),
        ));
    }
    return Ok(());
}

pub async fn oneshot(
    router: &Router,
    req: axum::extract::Request,
    expect_response: Response,
) -> Result<(), ValidationError> {
    let response = get_response(router, req).await?;
    validata_status_code(&response, &expect_response).await?;
    validate_json_body(response, expect_response).await?;
    Ok(())
}

static ONESHOT_PASS: Result<(), ValidationError> = Ok(());

pub async fn checked_oneshot(
    router: &Router,
    req: axum::extract::Request,
    expect_response: Response,
) {
    oneshot(router, req, expect_response).await.unwrap();
    //assert_eq!(result, ONESHOT_PASS);
}

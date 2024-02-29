use crate::state::SharedState;

use super::{error::AuthError, TokenParser};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
    iss: String, // Optional. Issuer
    nbf: usize, // Optional. Not Before (as UTC timestamp)
    sub: String, // Optional. Subject (whom token refers to)
    user_id: String,
    auth_id: String,
}

impl Claims {
    pub fn new<'a>(
        user_id: impl Into<Cow<'a, str>>,
        auth_id: impl Into<Cow<'a, str>>,
        ttl: Duration,
    ) -> Self {
        let now = Utc::now().timestamp_millis() as usize;
        let expire = now + ttl.num_milliseconds() as usize;
        let user_id = user_id.into().to_string();
        let auth_id = auth_id.into().to_string();

        let claims = Claims {
            aud: "".to_owned(),
            exp: expire,
            iat: now,
            iss: "".to_owned(),
            nbf: now,
            sub: "".to_owned(),
            user_id,
            auth_id,
        };

        return claims;
    }

    pub async fn from_request(parts: &mut Parts, parser: &TokenParser) -> Result<Self, AuthError> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        // Decode the user data
        let claims: Claims = parser.decode(bearer.token())?;
        return Ok(claims);
    }
}

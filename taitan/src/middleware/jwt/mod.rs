use axum::RequestPartsExt;
use axum::{
    async_trait,
    extract::ConnectInfo,
    extract::FromRequestParts,
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::TypedHeader;
use chrono::Utc;

use headers::{authorization::Bearer, Authorization};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::net::SocketAddr;
use tracing::info;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;

mod authorizator;
mod blacklist;
mod claims;
mod error;
mod keys;
mod token_parser;
use keys::KEYS;

pub use token_parser::TokenParser;

/*
#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

async fn authorize(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> {
    // Check if the user sent the credentials
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    // Here, basic verification is used but normally you would use a database
    if &payload.client_id != "foo" || &payload.client_secret != "bar" {
        return Err(AuthError::WrongCredentials);
    }

    // create the timestamp for the expiry time - here the expiry time is 1 day
    // in production you may not want to have such a long JWT life
    let exp = (Utc::now().naive_utc() + chrono::naive::Days::new(1)).timestamp() as usize;
    let claims = Claims {
        username: payload.client_id,
        exp,
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}

pub async fn authenticate<B>(req: Request<B>, next: Next) -> Result<Response, StatusCode> {
    dbg!(&req.extensions());
    let connect_info = req.extensions().get::<ConnectInfo<SocketAddr>>();
    dbg!(&connect_info);
    dbg!(&req.headers());
    // let http_info = req.extensions().get::<HttpInfo>();

    if not_need_authenticate(req.method().as_str(), req.uri().path()) {
        return Ok(next.run(req).await);
    }

    // running extractors requires a `axum::http::request::Parts`
    let (mut parts, body) = req.into_parts();

    // `TypedHeader<Authorization<Bearer>>` extracts the auth token
    let auth_token: TypedHeader<Authorization<Bearer>> = parts
        .extract()
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !token_is_valid(auth_token.token()) {
        info!(auth = "pass");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let request = Request::from_parts(parts, body);
    return Ok(next.run(request).await);
}

fn not_need_authenticate(method: &str, url: &str) -> bool {
    println!("url:{}", url);
    if method == "POST" && url == "/token" {
        return true;
    }
    if method == "GET" {
        if url.eq("/") {
            return true;
        }
        if url.starts_with("/assets/") || url.ends_with(".html") || url.ends_with("./ico") {
            return true;
        }
    }
    return false;
}

fn token_is_valid(token: &str) -> bool {
    println!("check if {token} is valid");
    // successful decode and not expire
    let token_data_result = decode::<Claims>(token, &KEYS.decoding, &Validation::default());
    if token_data_result.is_err() {
        return false;
    }
    // not in blacklist
    return token_not_in_blacklist(token);
}

fn token_not_in_blacklist(_token: &str) -> bool {
    return true;
}

pub async fn authorize(Json(payload): Json<AuthPayload>) -> Result<Json<AuthBody>, AuthError> {
    // Check if the user sent the credentials
    if payload.user_id.is_empty() || payload.user_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    is_valid_user(&payload.user_id, &payload.user_secret).await?;

    let claims = Claims {
        sub: "b@b.com".to_owned(),
        company: "ACME".to_owned(),
        // Mandatory expiry time as UTC timestamp
        exp: 2000000000, // May 2033
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(Json(AuthBody::new(token)))
}

// check the user credentials from a database
async fn is_valid_user(user_id: &str, user_secret: &str) -> Result<bool, AuthError> {
    if user_id != "foo" || user_secret != "bar" {
        return Err(AuthError::WrongCredentials);
    } else {
        return Ok(true);
    }
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email: {}\nCompany: {}", self.sub, self.company)
    }
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| AuthError::InvalidToken)?;

        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    company: String,
    exp: usize,
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    user_id: String,
    user_secret: String,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

*/

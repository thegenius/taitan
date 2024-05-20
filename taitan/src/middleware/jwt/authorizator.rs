use chrono::Duration;

use super::error::AuthError;
use std::borrow::Cow;

use super::{claims::Claims, TokenParser};

pub struct PathWhiteList {
    paths: Vec<String>,
}

impl PathWhiteList {
    pub fn new() -> Self {
        Self { paths: Vec::new() }
    }

    pub fn add_path<'a>(&mut self, path: impl Into<Cow<'a, str>>) {
        self.paths.push(path.into().to_string());
    }

    pub fn contains<'a>(&self, path: impl Into<Cow<'a, str>>) -> bool {
        self.paths.contains(&path.into().to_string())
    }
}

pub struct TokenBlacklist {
    tokens_buff: [Vec<String>; 2],
    index: usize,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            tokens_buff: [Vec::new(), Vec::new()],
            index: 0,
        }
    }

    pub fn add_token<'a>(&mut self, token: impl Into<Cow<'a, str>>) {
        self.tokens_buff[self.index].push(token.into().to_string());
    }

    pub fn del_token<'a>(&mut self, token: impl Into<Cow<'a, str>>) {
        self.tokens_buff[self.index].push(token.into().to_string());
    }
}

pub struct AuthorizationInfo {
    user_id: String,
    auth_id: String,
}

pub struct Authorizator {
    whitelist: PathWhiteList,
    parser: TokenParser,
    ttl: Duration,
    blacklist: TokenBlacklist,
}

pub enum AuthorizationResponse {
    Success(Claims),
    NotNeed,
    Failure(String),
}

impl Authorizator {
    pub fn authorize<'a>(
        &self,
        user_id: impl Into<Cow<'a, str>>,
        auth_id: impl Into<Cow<'a, str>>,
    ) -> Result<String, AuthError> {
        let claims: Claims = Claims::new(user_id, auth_id, self.ttl);
        let token = self.parser.encode(&claims)?;
        return Ok(token);
    }

    pub fn validate<'a>(
        &self,
        path: impl Into<Cow<'a, str>>,
        token: impl Into<Cow<'a, str>>,
    ) -> AuthorizationResponse {
        if self.whitelist.contains(path) {
            return AuthorizationResponse::NotNeed;
        }

        let claims: Result<Claims, AuthError> = self.parser.decode(token);
        return match claims {
            Ok(claims) => AuthorizationResponse::Success(claims),
            Err(err) => AuthorizationResponse::Failure(err.to_string()),
        };
    }
}

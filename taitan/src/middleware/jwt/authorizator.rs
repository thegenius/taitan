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

pub struct AuthorizationInfo {
    user_id: String,
    auth_id: String,
}

pub struct Authorizator {
    whitelist: PathWhiteList,
    parser: TokenParser,
}

pub enum AuthorizationResponse {
    Success(Claims),
    NotNeed,
    Failure(String),
}

impl Authorizator {
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

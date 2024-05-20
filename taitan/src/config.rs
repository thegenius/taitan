use serde::{Deserialize, Serialize};
use std::borrow::Cow;

const DEFAULT_MAX_BODY_LIMIT: usize = 10 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Args<'a> {
    pub http: HttpConfig<'a>,
    pub statics: Option<StaticFilesConfig<'a>>,
    pub log_dir: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StaticFilesConfig<'a> {
    pub assets_dir: Cow<'a, str>,
    pub assets_uri: Cow<'a, str>,
}
impl<'a> StaticFilesConfig<'a> {
    pub fn new(
        assets_dir: impl Into<Cow<'a, str>>,
        assets_uri: impl Into<Cow<'a, str>>,
    ) -> StaticFilesConfig<'a> {
        Self {
            assets_dir: assets_dir.into(),
            assets_uri: assets_uri.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HttpConfig<'a> {
    pub domain: Cow<'a, str>,
    pub port: u16,
    pub tls: Option<TlsConfig<'a>>,
    pub max_body_limit: usize,
}
impl<'a> HttpConfig<'a> {
    pub fn new(
        domain: impl Into<Cow<'a, str>>,
        port: u16,
        tls: Option<TlsConfig<'a>>,
        max_body_limit: Option<usize>
    ) -> HttpConfig<'a> {
        Self {
            domain: domain.into(),
            port,
            tls,
            max_body_limit: max_body_limit.unwrap_or(DEFAULT_MAX_BODY_LIMIT)
        }
    }
    pub fn from(
        domain: impl Into<Cow<'a, str>>,
        pem_file: impl Into<Cow<'a, str>>,
        key_file: impl Into<Cow<'a, str>>,
    ) -> HttpConfig<'a> {
        Self {
            domain: domain.into(),
            port: 80,
            tls: Some(TlsConfig::new(443, pem_file, key_file)),
            max_body_limit: DEFAULT_MAX_BODY_LIMIT
        }
    }
    pub fn from_domain(domain: impl Into<Cow<'a, str>>) -> HttpConfig<'a> {
        Self {
            domain: domain.into(),
            port: 80,
            tls: None,
            max_body_limit: DEFAULT_MAX_BODY_LIMIT
        }
    }
    pub fn local() -> HttpConfig<'a> {
        Self {
            domain: "localhost".into(),
            port: 80,
            tls: None,
            max_body_limit: DEFAULT_MAX_BODY_LIMIT
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TlsConfig<'a> {
    pub port: u16,
    pub pem_file: Cow<'a, str>,
    pub key_file: Cow<'a, str>,
}

impl<'a> TlsConfig<'a> {
    pub fn new(
        port: u16,
        pem_file: impl Into<Cow<'a, str>>,
        key_file: impl Into<Cow<'a, str>>,
    ) -> TlsConfig<'a> {
        Self {
            port,
            pem_file: pem_file.into(),
            key_file: key_file.into(),
        }
    }
}
/*
fn build_default_log_dir() -> String {
    let current_path = std::env::current_dir().unwrap();
    let log_dir = current_path.join("logs");
    return log_dir.to_str().unwrap().to_string();
}
*/

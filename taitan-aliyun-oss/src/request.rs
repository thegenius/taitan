use super::utils::{canonical, trim};
use derive_builder::Builder;
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone)]
pub enum HttpProtocol {
    Http,
    Https,
}

#[derive(Debug, Serialize, Clone)]
pub struct Entry(pub String, pub String);

#[derive(Debug, Clone, Builder)]
pub struct HttpRequest<T>
where
    T: Into<Body>,
{
    pub protocol: HttpProtocol,
    pub host: String,
    pub url: String,
    pub queries: Vec<Entry>,
    pub headers: Vec<Entry>,
    pub body: Option<T>,
}

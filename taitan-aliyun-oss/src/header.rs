use super::utils::{canonical, trim};
use std::borrow::Cow;

pub struct HeaderEntry<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> HeaderEntry<'a> {
    pub fn new(name: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> HeaderEntry<'a> {
        Self::with_value(name, value)
    }

    pub fn with_values<T>(name: impl Into<Cow<'a, str>>, values: impl AsRef<[T]>) -> HeaderEntry<'a>
    where
        T: AsRef<str>,
    {
        let value = values
            .as_ref()
            .iter()
            .map(|e| e.as_ref().trim())
            .collect::<Vec<&str>>()
            .join(",");
        Self {
            name: canonical(name),
            value: trim(value),
        }
    }

    pub fn with_value(
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
    ) -> HeaderEntry<'a> {
        Self {
            name: canonical(name),
            value: trim(value),
        }
    }

    pub fn format(&self) -> String {
        format!("{}:{}\n", self.name, self.value)
    }
}

// all the heades that start with "x-acs-" and "host"
pub struct CanonicalHeaders<'a> {
    entries: Vec<HeaderEntry<'a>>,
}

impl<'a> CanonicalHeaders<'a> {
    pub fn format(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.format())
            .collect::<Vec<String>>()
            .join("")
    }
    pub fn signed_headers(&self) -> String {
        self.entries
            .iter()
            .map(|e| e.name.to_string())
            .collect::<Vec<String>>()
            .join(";")
    }
}

pub enum Action {
    AssumeRole,
}

pub struct RequestHeader {
    // example: oss-us-west-1.aliyuncs.com
    host: String,

    // example: AssumeRole
    x_acs_action: String,

    // example: 2015-04-01
    x_acs_version: String,

    // ISO-8601 format: 2018-01-01T12:00:00Z
    x_acs_date: String,

    // Hex(HashMac( Request Payload ))
    x_acs_content_sha256: String,

    // random value
    x_acs_signature_nonce: Option<String>,

    // sts request must set
    // check it from AssumeRole's SecurityToken
    x_acs_security_token: Option<String>,

    Authorization: String,
}

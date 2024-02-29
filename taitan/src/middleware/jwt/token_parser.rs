use super::{error::AuthError, keys::Key};
use jsonwebtoken::{decode, encode, Header, TokenData, Validation};
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;

pub struct TokenParser {
    key: Key,
}

impl Default for TokenParser {
    fn default() -> Self {
        Self {
            key: Key::default(),
        }
    }
}

impl TokenParser {
    pub fn encode<T: Serialize>(&self, data: &T) -> Result<String, AuthError> {
        let token = encode(&Header::default(), data, &self.key.encoding)?;
        return Ok(token);
    }

    pub fn decode<'a, T: DeserializeOwned>(
        &self,
        token: impl Into<Cow<'a, str>>,
    ) -> Result<T, AuthError> {
        let data: TokenData<T> = decode::<T>(
            token.into().as_ref(),
            &self.key.decoding,
            &Validation::default(),
        )?;
        return Ok(data.claims);
    }
}

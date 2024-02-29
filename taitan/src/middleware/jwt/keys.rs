use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};

pub struct Key {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Default for Key {
    fn default() -> Self {
        let secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 60);
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
}

impl Key {
    pub fn new(secret: impl AsRef<[u8]>) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_ref()),
            decoding: DecodingKey::from_secret(secret.as_ref()),
        }
    }
}

pub static KEYS: Lazy<Key> = Lazy::new(|| {
    let secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 60);
    Key::new(secret.as_bytes())
});

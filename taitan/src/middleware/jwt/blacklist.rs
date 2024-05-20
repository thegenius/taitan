use std::borrow::Cow;

use super::error::AuthError;
use std::time::Duration;
use tokio::sync::RwLock;

pub trait BlackListLoader {
    async fn load(&self) -> Result<Vec<String>, AuthError>;
}

pub struct TokenBlacklist<T: BlackListLoader + Sync> {
    tokens: RwLock<Vec<String>>,
    loader: T,
    loader_interval_secs: usize,
}

impl<T> TokenBlacklist<T>
where
    T: BlackListLoader + Sync,
{
    pub fn new(loader: T, interval: usize) -> Self {
        let black = TokenBlacklist {
            tokens: RwLock::new(Vec::new()),
            loader,
            loader_interval_secs: interval,
        };
        // black.interval_load();
        return black;
    }

    // pub fn interval_load(&self) {
    //     tokio::spawn(async move {
    //         loop {
    //             let duration = Duration::from_secs(self.loader_interval_secs as u64);
    //             tokio::time::sleep(duration).await;
    //             let tokens = self.loader.load().await.unwrap_or(Vec::new());
    //             let mut token = self.tokens.write().await;
    //             *token = tokens;
    //         }
    //     });
    // }

    pub async fn add_token<'a>(&mut self, token: impl Into<Cow<'a, str>>) {
        self.tokens.write().await.push(token.into().to_string());
    }

    pub async fn del_token<'a>(&mut self, token: impl Into<Cow<'a, str>>) {
        self.tokens.write().await.push(token.into().to_string());
    }
}

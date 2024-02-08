use crate::request::HttpRequest;
use reqwest::Body;
use reqwest::Client;
use reqwest::Method;
use reqwest::Response;
use std::time::Duration;
use taitan::result::Result;

pub struct OssClient {
    timeout: Duration,
    client: Client,
}

impl OssClient {
    pub fn new(timeout: Duration) -> Self {
        let client = Client::new();
        Self { client, timeout }
    }

    pub async fn get<T>(&self, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        self.execute(Method::GET, req).await
    }

    pub async fn post<T>(&self, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        self.execute(Method::POST, req).await
    }

    pub async fn delete<T>(&self, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        self.execute(Method::DELETE, req).await
    }

    pub async fn put<T>(&self, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        self.execute(Method::PUT, req).await
    }

    pub async fn head<T>(&self, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        self.execute(Method::HEAD, req).await
    }

    pub async fn execute<T>(&self, method: Method, req: HttpRequest<T>) -> Result<Response>
    where
        T: Into<Body>,
    {
        let mut builder = self.client.request(method, req.url);
        builder = builder.timeout(self.timeout);

        builder = builder.query(&req.queries);
        for entry in req.headers {
            builder = builder.header(entry.0, entry.1);
        }
        if let Some(body) = req.body {
            builder = builder.body(body);
        }
        let response = builder.send().await?;
        Ok(response)
    }
}

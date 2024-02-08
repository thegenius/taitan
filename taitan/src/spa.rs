use axum::{
    extract::Request, handler::HandlerWithoutStateExt, http::StatusCode, routing::get, Router,
};
use std::future::Future;
use std::net::SocketAddr;
use std::{borrow::Cow, convert::Infallible};
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

fn get_spa_router<'a, I>(dist: impl Into<Cow<'a, str>>, index: Option<I>) -> Router
where
    I: tower::Service<Request, Error = Infallible> + Send + Clone + 'static,
    I::Response: axum::response::IntoResponse,
    I::Future: Send + 'static,
{
    let static_path = dist.into();
    let assets_dir = std::path::Path::new(static_path.as_ref()).join("assets");
    let serve_dir = ServeDir::new(assets_dir);
    if let Some(index) = index {
        Router::new()
            .nest_service("/assets", serve_dir)
            .fallback_service(index)
    } else {
        let index_path = std::path::Path::new(static_path.as_ref()).join("index.html");
        let index = ServeFile::new(index_path);
        Router::new()
            .nest_service("/assets", serve_dir)
            .fallback_service(index)
    }
}

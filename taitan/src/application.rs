use crate::config::HttpConfig;
use crate::config::{self, Args, StaticFilesConfig};
use axum::Router;

use axum::handler::HandlerWithoutStateExt;
use axum::{
    extract::Host,
    http::{StatusCode, Uri},
    response::Redirect,
    routing::{get, post},
    BoxError,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use tokio::signal;
use tokio::time::sleep;
use tracing::info;

pub struct Application<'a> {
    router: Router,
    args: config::Args<'a>,
}

async fn ok() -> StatusCode {
    return StatusCode::OK;
}

fn get_default_router() -> Router {
    let router = Router::new()
        .route("/", get(ok))
        .route("/health", get(ok))
        .route("/ready", get(ok));
    return router;
}

impl<'a> Application<'a> {
    pub fn new(router: Router, args: config::Args<'a>) -> Self {
        Self { router, args }
    }

    pub fn default_dev() -> Self {
        let args = Args {
            http: HttpConfig::new("0.0.0.0", 80, None),
            statics: None,
            log_dir: None,
        };
        Self {
            router: get_default_router(),
            args,
        }
    }

    #[cfg(debug_assertions)]
    pub async fn run(self) {
        self.run_http().await;
    }

    #[cfg(not(debug_assertions))]
    pub async fn run(self) {
        self.run_https().await;
    }

    pub async fn run_https(self) {
        let tls_config = self
            .args
            .http
            .tls
            .as_ref()
            .expect("tls config must be set when running https server.");

        info!(name = "TAITAN", "start https server ...");
        let http_port = self.args.http.port.to_string();
        let https_port = tls_config.port.to_string();
        tokio::spawn(make_redirect_server(
            self.args.http.port,
            http_port,
            https_port,
        ));
        make_https_server(self.router, self.args.http).await;
    }

    pub async fn run_http(self) {
        info!(name = "TAITAN", "start http server ...");
        info!(name = "TAITAN", config = ?self.args);
        let http_config = self.args.http;
        make_http_server(self.router, http_config).await;
    }
}

#[cfg(debug_assertions)]
async fn make_http_server<'a>(router: Router, http_config: HttpConfig<'a>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], http_config.port));
    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone()));
    axum_server::bind(addr)
        .handle(handle)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

//#[cfg(not(debug_assertions))]
async fn make_https_server<'a>(router: Router, http_config: HttpConfig<'a>) {
    let tls_config = http_config.tls.expect("tls config must be set.");
    let pem_file = PathBuf::from(tls_config.pem_file.as_ref());
    let key_file = PathBuf::from(tls_config.key_file.as_ref());
    let rustls_config = RustlsConfig::from_pem_file(pem_file, key_file)
        .await
        .expect("pem file or key file not found");

    let addr = SocketAddr::from(([0, 0, 0, 0], tls_config.port));
    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone()));
    axum_server::bind_rustls(addr, rustls_config)
        .handle(handle)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

//#[cfg(not(debug_assertions))]
pub async fn make_redirect_server<'a>(port: u16, http_port: String, https_port: String) {
    fn make_https(
        host: String,
        uri: Uri,
        http_port: String,
        https_port: String,
    ) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();
        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(http_port.as_str(), https_port.as_str());
        parts.authority = Some(https_host.parse()?);
        Ok(Uri::from_parts(parts)?)
    }

    let ipv4_addr: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    let addr = SocketAddr::new(IpAddr::V4(ipv4_addr), port);

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, http_port, https_port) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    tracing::debug!("http redirect listening on {}", addr);

    axum_server::bind(addr)
        .serve(redirect.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn graceful_shutdown(handle: Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!(
        name = "TAITAN",
        "signal received, starting graceful shutdown, deadline is 2000ms"
    );
    handle.graceful_shutdown(Some(Duration::from_millis(2000)));
    loop {
        sleep(Duration::from_millis(1000)).await;
        info!(
            name = "TAITAN",
            "start graceful shutdown, alive connections: {}",
            handle.connection_count()
        );
    }
}

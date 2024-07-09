

use axum::Router;
use axum::handler::HandlerWithoutStateExt;
use axum::{
    extract::DefaultBodyLimit,
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
use std::fs::{create_dir_all, File, metadata};
use clap::Parser;
use crate::daemonize::get_daemonize;

use tokio::runtime::Runtime;
use tokio::runtime::Builder;
use std::path::Path;
use crate::server_conf::{Args, ApplicationConfig, HttpConfig, TlsConfig};

pub struct Application {
    router: Router,
    // args: config::Args,
    args: ApplicationConfig
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

pub enum RunningMode {
    Http,
    Https,
    HttpDaemon,
    HttpsDaemon,
}

impl Application {
    pub fn new(router: Router, args: ApplicationConfig) -> Self {
        Self { router, args }
    }

    pub fn default_dev() -> Self {
        Self {
            router: get_default_router(),
            args: ApplicationConfig::default(),
        }
    }

    pub fn from_args(router: Router) -> Self {
        let args = Args::parse();
        Self::ensure_folder_exists(&args.workspace).unwrap();
        let config_path = Path::new(&args.workspace).join(&args.application_config_file);
        if !Self::file_or_dir_exists(&config_path) {
            let mut default_args = ApplicationConfig::default();
            let workspace_path = std::fs::canonicalize(args.workspace).unwrap();
            default_args.workspace = workspace_path.as_os_str().to_string_lossy().to_string();
            let default_args_string = toml::to_string(&default_args).unwrap();
            std::fs::write(&config_path, default_args_string).unwrap();
        }
        let config_toml = std::fs::read_to_string(config_path).unwrap();
        let args = ApplicationConfig::from_toml(config_toml);

        Self {
            router,
            args
        }
    }

    fn file_or_dir_exists<P: AsRef<Path>>(path: &P) -> bool {
        std::fs::metadata(path).is_ok()
    }

    fn ensure_folder_exists(path: &str) -> Result<(), std::io::Error> {
        let path = Path::new(path);
        if !path.exists() || !metadata(path)?.is_dir() {
            create_dir_all(path)?;
        }
        Ok(())
    }


    pub fn get_router(&self) -> &Router {
        return &self.router;
    }

    pub fn add_router(&mut self, router: Router) {
        let merged = Router::new().merge(self.router.clone()).merge(router);
        self.router = merged;
    }

    pub async fn run(self, mode: RunningMode) {
        match mode {
            RunningMode::Http => {
                self.run_http().await;
            }
            RunningMode::Https => {
                self.run_https().await;
            }
            RunningMode::HttpDaemon => {
                let daemonize = get_daemonize(&self.args.daemon);
                match daemonize.start() {
                    Ok(_) => {
                        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
                        rt.block_on(async {
                            self.run_http().await;
                        })
                    }
                    Err(e) => eprintln!("Error, {}", e),
                }
            }
            RunningMode::HttpsDaemon => {
                let daemonize = get_daemonize(&self.args.daemon);
                match daemonize.start() {
                    Ok(_) => {
                        let rt = Builder::new_multi_thread().enable_all().build().unwrap();
                        rt.block_on(async {
                            self.run_https().await;
                        })
                    }
                    Err(e) => eprintln!("Error, {}", e),
                }
            }
        }
    }


    pub async fn run_https(self) {
        let tls_config = self
            .args
            .http
            .tls
            .as_ref()
            .expect("tls config must be set when running https server.");

        info!(name = "TAITAN", "start https server ...");
        let http_port = self.args.http.http_port.to_string();
        let https_port = tls_config.https_port.to_string();
        tokio::spawn(make_redirect_server(
            self.args.http.http_port,
            http_port,
            https_port,
        ));
        make_https_server(self.router, self.args).await;
    }

    pub async fn run_http(self) {
        info!(name = "TAITAN", "start http server ...");
        info!(name = "TAITAN", config = ?self.args);
        let http_config = self.args.http;
        make_http_server(self.router, http_config).await;
    }
}

async fn make_http_server<'a>(router: Router, http_config: HttpConfig) {
    let addr = SocketAddr::from(([0, 0, 0, 0], http_config.http_port));

    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone()));
    axum_server::bind(addr)
        .handle(handle)
        .serve(
            router
                .layer(DefaultBodyLimit::max(http_config.max_body_size))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
}

async fn make_https_server<'a>(router: Router, app_config: ApplicationConfig) {
    let tls_config = app_config.http.tls.expect("tls config must be set.");
    let pem_file = PathBuf::from(Path::new(&app_config.workspace).join(tls_config.tls_crt));
    let key_file = PathBuf::from(Path::new(&app_config.workspace).join(tls_config.tls_key));
    let rustls_config = RustlsConfig::from_pem_file(pem_file, key_file)
        .await
        .expect("pem file or key file not found");

    let addr = SocketAddr::from(([0, 0, 0, 0], tls_config.https_port));
    let handle = Handle::new();
    tokio::spawn(graceful_shutdown(handle.clone()));
    axum_server::bind_rustls(addr, rustls_config)
        .handle(handle)
        .serve(
            router
                .layer(DefaultBodyLimit::max(app_config.http.max_body_size))
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
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

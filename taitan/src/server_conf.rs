use clap::{Parser};
use serde::{Serialize, Deserialize};
use serde_inline_default::serde_inline_default;
use derive_new::new;
use derive_builder::Builder;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value = "./workspace/")]
    pub workspace: String,

    #[arg(short, long, default_value = "application.toml")]
    pub application_config_file: String
}



#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct HttpConfig {
    #[serde_inline_default(80)]
    pub http_port: u16,

    #[serde_inline_default(10 * 1024 * 1024)]
    pub max_body_size: usize,

    #[serde_inline_default(None)]
    pub tls: Option<TlsConfig>,
}


#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct TlsConfig {

    #[serde_inline_default(false)]
    pub redirect_to_https: bool,

    #[serde_inline_default(443)]
    pub https_port: u16,

    #[serde_inline_default("keys/tls_key.pem".to_string())]
    pub tls_key: String,

    #[serde_inline_default("keys/tls_crt.pem".to_string())]
    pub tls_crt: String,
}


#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct JwtConfig {
    #[serde_inline_default("keys/jwt_key.pem".to_string())]
    pub key: String,

    #[serde_inline_default("keys/jwt_crt.pem".to_string())]
    pub crt: String,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct AssetsConfig {
    #[serde_inline_default("assets/".to_string())]
    pub asserts_uri: String,

    #[serde_inline_default("assets/".to_string())]
    pub asserts_dir: String,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct LogsConfig {
    #[serde_inline_default("logs/".to_string())]
    pub logs_dir: String,

}

#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Default, Clone, Debug)]
pub struct DaemonConfig {

    #[serde_inline_default(false)]
    pub daemon: bool,

    #[serde_inline_default("daemon/app.pid".to_string())]
    pub pid_file: String,

    #[serde_inline_default(Some("daemon/error.log".to_string()))]
    pub error_log: Option<String>,

    #[serde_inline_default(None)]
    pub user: Option<String>,

    #[serde_inline_default(None)]
    pub group: Option<String>,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize, new, Builder, Clone, Debug)]
pub struct ApplicationConfig {
    #[serde_inline_default("./workspace".to_string())]
    pub workspace: String,

    #[serde_inline_default(HttpConfig::default())]
    pub http: HttpConfig,

    #[serde_inline_default(JwtConfig::default())]
    pub jwt: JwtConfig,

    #[serde_inline_default(AssetsConfig::default())]
    pub asset: AssetsConfig,

    #[serde_inline_default(LogsConfig::default())]
    pub logs: LogsConfig,

    #[serde_inline_default(DaemonConfig::default())]
    pub daemon: DaemonConfig,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            workspace: "./workspace".to_string(),
            http: toml::from_str("").unwrap(),
            jwt: toml::from_str("").unwrap(),
            asset: toml::from_str("").unwrap(),
            logs: toml::from_str("").unwrap(),
            daemon: toml::from_str("").unwrap(),
        }
    }
}

impl ApplicationConfig {
    pub fn from_toml<T: AsRef<str>>(content: T) -> Self {
        let data = content.as_ref();
        if data.is_empty() {
            Self::default()
        } else {
            toml::from_str(content.as_ref()).unwrap()
        }
    }
}

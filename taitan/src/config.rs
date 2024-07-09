// use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
// use clap::{Parser, Subcommand};
// use derive_new::new;
//
// const DEFAULT_MAX_BODY_LIMIT: usize = 10 * 1024 * 1024;
//
// // #[derive(Parser)]
// // #[command(version, about, long_about = None)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct Args {
//     // #[command(subcommand)]
//     pub http: HttpConfig,
//
//     // #[command(subcommand)]
//     pub statics: Option<StaticFilesConfig>,
//
//     // #[arg(long)]
//     pub log_dir: Option<String>,
// }
//
//
// #[derive(Parser)]
// #[derive(new)]
// #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
// pub struct StaticFilesConfig {
//     #[arg(long)]
//     pub assets_dir: String,
//
//     #[arg(long)]
//     pub assets_uri: String,
// }
//
// // #[derive(Subcommand)]
// #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
// pub struct HttpConfig {
//     // #[arg(long)]
//     pub domain: String,
//
//     // #[arg(long)]
//     pub port: u16,
//
//     // #[command(subcommand)]
//     pub tls: Option<TlsConfig>,
//
//     // #[arg(long)]
//     pub max_body_limit: usize,
// }
// impl HttpConfig {
//     pub fn new(
//         domain: impl Into<String>,
//         port: u16,
//         tls: Option<TlsConfig>,
//         max_body_limit: Option<usize>,
//     ) -> HttpConfig {
//         Self {
//             domain: domain.into(),
//             port,
//             tls,
//             max_body_limit: max_body_limit.unwrap_or(DEFAULT_MAX_BODY_LIMIT),
//         }
//     }
//     pub fn from(
//         domain: impl Into<String>,
//         pem_file: impl Into<String>,
//         key_file: impl Into<String>,
//     ) -> HttpConfig {
//         Self {
//             domain: domain.into(),
//             port: 80,
//             tls: Some(TlsConfig::new(443, pem_file, key_file)),
//             max_body_limit: DEFAULT_MAX_BODY_LIMIT,
//         }
//     }
//     pub fn from_domain(domain: impl Into<String>) -> HttpConfig {
//         Self {
//             domain: domain.into(),
//             port: 80,
//             tls: None,
//             max_body_limit: DEFAULT_MAX_BODY_LIMIT,
//         }
//     }
//     pub fn local() -> HttpConfig {
//         Self {
//             domain: "localhost".into(),
//             port: 80,
//             tls: None,
//             max_body_limit: DEFAULT_MAX_BODY_LIMIT,
//         }
//     }
// }
//
// // #[derive(Subcommand)]
// #[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
// pub struct TlsConfig {
//     // #[arg(long)]
//     pub https_port: u16,
//
//     // #[arg(long)]
//     pub pem_file: String,
//
//     // #[arg(long)]
//     pub key_file: String,
// }
//
// impl TlsConfig {
//     pub fn new(
//         https_port: u16,
//         pem_file: impl Into<String>,
//         key_file: impl Into<String>,
//     ) -> TlsConfig {
//         Self {
//             https_port,
//             pem_file: pem_file.into(),
//             key_file: key_file.into(),
//         }
//     }
// }

mod update_dns_txt;
mod acme;
mod tls;

use std::{env};
use std::io::Read;
use clap::Parser;
use rand::Rng;
use crate::update_dns_txt::{AliyunConfig, DnsConfig};
use rustls::crypto::aws_lc_rs::default_provider;


use axum::{
    handler::HandlerWithoutStateExt,
    routing::get,
    Router,
};

use std::{net::SocketAddr};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::tls::gen_rustls_config;

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}



#[allow(dead_code)]
async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();
    let provider = default_provider();
    provider
        .install_default()
        .expect("Failed to set CryptoProvider");

    // let random_value  = rand::thread_rng().gen_range(0..u64::MAX).to_string();

    let mut update_request = DnsConfig::new(
       "lvonce.com".to_string(),          // 主域名，如 "example.com"
        "_acme-challenge".to_string(),  // 子域名，如 "_acme-challenge"
    );

    let config = AliyunConfig {
        access_key_id: env::var("ACCESS_KEY_ID")?,
        access_key_secret: env::var("ACCESS_KEY_SECRET")?,
    };


    let config = gen_rustls_config(&config, &mut update_request).await?;

    let app = Router::new().route("/", get(handler));

    // run https server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8443));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}


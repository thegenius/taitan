mod update_dns_txt;
mod acme;
mod tls;

use std::{env, io, time::Duration};
use std::io::Read;
use clap::Parser;
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use tokio::time::sleep;
use tracing::{error, info};

use instant_acme::{Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt, NewAccount, NewOrder, OrderStatus};
use rand::Rng;
use crate::acme::gen_tls_pem;
use crate::update_dns_txt::{AliyunConfig, UpdateDnsTxtRequest};




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    // let random_value  = rand::thread_rng().gen_range(0..u64::MAX).to_string();

    let mut update_request = UpdateDnsTxtRequest::new(
       "lvonce.com".to_string(),          // 主域名，如 "example.com"
        "_acme-challenge".to_string(),  // 子域名，如 "_acme-challenge"
    );

    let config = AliyunConfig {
        access_key_id: env::var("ACCESS_KEY_ID")?,
        access_key_secret: env::var("ACCESS_KEY_SECRET")?,
    };

    let tls_pem = gen_tls_pem(&config, &mut update_request, "default.acme").await?;

    // if let Err(err) = update_dns_txt::update_dns_txt_record(&config, &update_request).await {
    //     info!("update_dns_txt error: {err}");
    // }

    // Create a new account. This will generate a fresh ECDSA key for you.
    // Alternatively, restore an account from serialized credentials by
    // using `Account::from_credentials()`.
    // let acme_file_path = "default.acme";
    // let credentials = get_acme_credentials(acme_file_path.to_string()).await?;
    // let tls_pem = gen_tls_pem(credentials).await?;

    Ok(())
}


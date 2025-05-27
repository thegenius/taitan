use crate::acme::{TslPem, gen_tls_pem};
use crate::update_dns_txt::{AliyunConfig, DnsConfig};
use axum_server::tls_rustls::RustlsConfig;
use instant_acme::{Account, LetsEncrypt, NewAccount};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, ServerConfig};
use std::sync::Arc;
use tracing::info;

pub async fn gen_rustls_config(
    config: &AliyunConfig,
    request: &mut DnsConfig,
) -> anyhow::Result<RustlsConfig> {
    let tls_pem = match gen_tls_pem(&config, request).await {
        Ok(tls_pem) => tls_pem,
        Err(err) => {
            return Err(anyhow::anyhow!(err.to_string()));
        }
    };
    info!("TLS: {:?}", tls_pem);
    let config = gen_server_config(tls_pem)?;
    Ok(RustlsConfig::from_config(Arc::new(config)))
}

// async fn gen_rustls_config_from_cache(
//     config: &AliyunConfig,
//     request: &mut DnsConfig,
//     crt_file: String,
//     key_file: String,
// ) -> anyhow::Result<RustlsConfig> {
//     let mut acme_file_result = std::fs::File::open(crt_file);
//     match acme_file_result {
//         Err(err) => {
//             if err.kind() == std::io::ErrorKind::NotFound {
//                 info!("default.acme not found, creating new default.acme file");
//
//             } else {
//                 Err(err.into())
//             }
//         }
//         Ok(mut acme_file) => {
//             info!("default.acme found, loading from it.");
//             let mut contents = String::new();
//             acme_file.read_to_string(&mut contents)?;
//             let credentials: AccountCredentials = serde_json::from_str(&contents)?;
//             Ok(Account::from_credentials(credentials).await?)
//         }
//     }
// }

pub fn gen_server_config(tls_pem: TslPem) -> anyhow::Result<ServerConfig> {
    let certs = parse_cert_pem(&tls_pem.crt)?;
    let private_key = PrivateKeyDer::from_pem_slice(tls_pem.key.as_bytes())
        .map_err(|err| anyhow::anyhow!("could not load private key: {}", err))?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;
    Ok(config)
}

fn parse_cert_pem(pem_str: &str) -> anyhow::Result<Vec<CertificateDer<'static>>> {
    let mut certs: Vec<CertificateDer<'static>> = Vec::new();
    let certs_iter = CertificateDer::pem_slice_iter(pem_str.as_bytes());
    for cert in certs_iter {
        match cert {
            Ok(cert) => {
                certs.push(cert);
            }
            Err(_) => {
                return Err(anyhow::Error::msg("parse cert failed"));
            }
        }
    }
    Ok(certs)
}

use crate::update_dns_txt::{AliyunConfig, DnsConfig, update_dns_txt_record};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, LetsEncrypt,
    NewAccount, NewOrder, Order, OrderState, OrderStatus,
};
use rcgen::{CertificateParams, DistinguishedName, KeyPair};
use rustls::crypto::CryptoProvider;
use std::io;
use std::io::Read;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::fmt::format;

async fn get_acme_account_from_cache(acme_file_path: String) -> anyhow::Result<Account> {
    let mut acme_file_result = std::fs::File::open("default.acme");
    match acme_file_result {
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                info!("default.acme not found, creating new default.acme file");
                let (account, credentials) = Account::create(
                    &NewAccount {
                        contact: &[],
                        terms_of_service_agreed: true,
                        only_return_existing: false,
                    },
                    LetsEncrypt::Staging.url(),
                    None,
                )
                .await
                .expect("could not create new account");
                let credential_data = serde_json::to_string_pretty(&credentials)
                    .expect("credentials serialize to json error");
                std::fs::write(acme_file_path, &credential_data)?;
                info!("account credentials:\n\n{}", credential_data);
                Ok(Account::from_credentials(credentials).await?)
            } else {
                Err(err.into())
            }
        }
        Ok(mut acme_file) => {
            info!("default.acme found, loading from it.");
            let mut contents = String::new();
            acme_file.read_to_string(&mut contents)?;
            let credentials: AccountCredentials = serde_json::from_str(&contents)?;
            Ok(Account::from_credentials(credentials).await?)
        }
    }
}

async fn get_acme_account() -> anyhow::Result<Account> {
    info!("creating new account");
    let (account, _credentials) = Account::create(
        &NewAccount {
            contact: &[],
            terms_of_service_agreed: true,
            only_return_existing: false,
        },
        LetsEncrypt::Staging.url(),
        None,
    )
    .await
    .map_err(anyhow::Error::from)?;
    // let credential_data =
    //     serde_json::to_string_pretty(&credentials).expect("credentials serialize to json error");
    // info!("account credentials:\n\n{}", credential_data);
    // Ok(Account::from_credentials(credentials).await?)
    Ok(account)
}

#[derive(Debug)]
pub struct TslPem {
    pub crt: String,
    pub key: String,
}

async fn wait_order_ready(
    aliyun_config: &AliyunConfig,
    dns_config: &mut DnsConfig,
    order: &mut Order,
) -> anyhow::Result<()> {
    let state = order.state();
    info!("order state: {:#?}", state);
    if state.status == OrderStatus::Ready {
        return Ok(());
    }

    let authorizations = order.authorizations().await?;
    let mut challenges = Vec::with_capacity(authorizations.len());
    info!("challenges count: {}", authorizations.len());
    let mut challenge_txt_values: Vec<String> = Vec::new();
    for authz in &authorizations {
        match authz.status {
            AuthorizationStatus::Pending => {}
            AuthorizationStatus::Valid => continue,
            _ => todo!(),
        }

        // We'll use the DNS challenges for this example, but you could
        // pick something else to use here.

        let challenge = authz
            .challenges
            .iter()
            .find(|c| c.r#type == ChallengeType::Dns01)
            .ok_or_else(|| anyhow::anyhow!("no dns01 challenge found"))?;

        let Identifier::Dns(identifier) = &authz.identifier;

        let dns_txt_value = order.key_authorization(challenge).dns_value();
        info!("Please set the following DNS record then press the Return key:");
        info!("_acme-challenge.{} IN TXT {}", identifier, dns_txt_value);
        challenge_txt_values.push(dns_txt_value);
        challenges.push((identifier, &challenge.url));
    }
    update_dns_txt_record(aliyun_config, dns_config, &challenge_txt_values).await?;

    // Let the server know we're ready to accept the challenges.
    for (_, url) in &challenges {
        order.set_challenge_ready(url).await.unwrap();
    }

    // Exponentially back off until the order becomes ready or invalid.
    let delay = Duration::from_millis(200);
    for i in 0..100 {
        sleep(delay).await;
        let state = order
            .refresh()
            .await
            .map_err(|e| anyhow::anyhow!("order refresh error: {:?}", e))?;
        match state.status {
            OrderStatus::Ready => return Ok(()),
            OrderStatus::Invalid => {
                return Err(anyhow::anyhow!("invalid order status: {:?}", state));
            }
            _ => {}
        }
    }

    Err(anyhow::anyhow!("order not ready after 20,000 ms "))
}

pub async fn gen_tls_pem(
    aliyun_config: &AliyunConfig,
    request: &mut DnsConfig,
) -> anyhow::Result<TslPem> {
    let account = get_acme_account().await?;

    // info!("acme account: {:?}", account);

    info!("create ACME order ...");
    // Create the ACME order based on the given domain names.
    // Note that this only needs an `&Account`, so the library will let you
    // process multiple orders in parallel for a single account.
    let domain_name = format!("*.{}", request.domain);
    let identifiers = vec![
        Identifier::Dns(format!("*.{}", request.domain)), // 通配符域名
        Identifier::Dns(request.domain.clone()),          // 根域名
    ];
    let new_order = NewOrder {
        identifiers: &identifiers,
    };
    let mut order = account.new_order(&new_order).await?;
    info!("order created!");


    wait_order_ready(aliyun_config, request, &mut order).await?;

    // let mut names = Vec::with_capacity(challenges.len());
    // for (identifier, _) in challenges {
    //     names.push(identifier.to_owned());
    // }

    // If the order is ready, we can provision the certificate.
    // Use the rcgen library to create a Certificate Signing Request.

    let mut params = CertificateParams::new(vec![domain_name.clone(), request.domain.clone()])?;
    params.distinguished_name = DistinguishedName::new();
    let private_key = KeyPair::generate()?;
    let csr = params.serialize_request(&private_key)?;

    // Finalize the order and print certificate chain, private key and account credentials.

    order.finalize(csr.der()).await.unwrap();
    let cert_chain_pem = loop {
        match order.certificate().await.unwrap() {
            Some(cert_chain_pem) => break cert_chain_pem,
            None => sleep(Duration::from_secs(1)).await,
        }
    };

    info!("certficate chain:\n\n{}", cert_chain_pem);
    info!("private key:\n\n{}", private_key.serialize_pem());
    let tls_pem = TslPem {
        crt: cert_chain_pem,
        key: private_key.serialize_pem(),
    };
    Ok(tls_pem)
}

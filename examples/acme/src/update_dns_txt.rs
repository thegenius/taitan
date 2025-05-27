use crate::update_dns_txt;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sha1::Sha1;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::{Resolver, TokioAsyncResolver};
use urlencoding::encode;

type HmacSha1 = Hmac<Sha1>;

const ALIYUN_API_URL: &str = "https://alidns.aliyuncs.com";

#[derive(Debug)]
pub struct AliyunConfig {
    pub(crate) access_key_id: String,
    pub(crate) access_key_secret: String,
}

#[derive(Debug)]
pub struct DnsConfig {
    pub domain: String,
    pub sub_domain: String,
}
#[derive(Debug)]
pub struct TxtRecords(Vec<String>);

impl DnsConfig {
    pub fn new(domain: String, sub_domain: String) -> Self {
        Self { domain, sub_domain }
    }
}

pub async fn update_dns_txt_record<T: AsRef<str> + PartialEq>(
    config: &AliyunConfig,
    dns_config: &DnsConfig,
    txt_values: &[T],
) -> Result<()> {
    for txt_value in txt_values {
        if let Err(err) = update(&config, &dns_config, txt_value).await {
            error!("{}", err);
            return Err(anyhow!(err));
        };
    }

   check(&dns_config, txt_values).await?;
    clear(config, dns_config, txt_values).await?;
    Ok(())
}

fn gen_basic_request_param<'a>(access_key_id: &str) -> BTreeMap<&'static str, String> {
    let mut params: BTreeMap<&'static str, String> = BTreeMap::new();
    params.insert("Version", "2015-01-09".to_string());
    params.insert("Format", "JSON".to_string());
    params.insert("AccessKeyId", access_key_id.to_string());
    params.insert("SignatureMethod", "HMAC-SHA1".to_string());
    params.insert("SignatureVersion", "1.0".to_string());
    params
}

fn complete_param(access_key_secret: &str, params: &mut BTreeMap<&'static str, String>) {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    params.insert("Timestamp", timestamp);

    let nonce = rand::thread_rng().gen_range(0..u64::MAX).to_string();
    params.insert("SignatureNonce", nonce);

    let signature = gen_signature(&params, access_key_secret);
    params.insert("Signature", signature);
}

fn gen_signature(params: &BTreeMap<&'static str, String>, secret: &str) -> String {
    // 1. 排序并编码参数
    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // 2. 构造签名字符串
    let string_to_sign = format!("POST&%2F&{}", encode(&query));

    // 3. 计算 HMAC-SHA1
    let mut hmac = HmacSha1::new_from_slice(format!("{}&", secret).as_bytes())
        .expect("HMAC can take key of any size");
    hmac.update(string_to_sign.as_bytes());
    let signature = hmac.finalize().into_bytes();

    // 4. Base64编码
    general_purpose::STANDARD.encode(signature)
}

fn fill_update_param<'a>(
    access_key_secret: &str,
    params: &mut BTreeMap<&'static str, String>,
    domain: &str,
    sub_domain: &str,
    txt_value: &str,
) {
    params.insert("Action", "AddDomainRecord".to_string());
    params.insert("DomainName", domain.to_string());
    params.insert("RR", sub_domain.to_string());
    params.insert("Type", "TXT".to_string());
    params.insert("Value", txt_value.to_string());
    complete_param(access_key_secret, params)
}
fn fill_desc_param(
    access_key_secret: &str,
    params: &mut BTreeMap<&'static str, String>,
    domain: &str,
    sub_domain: &str,
) {
    params.insert("Action", "DescribeDomainRecords".to_string());
    params.insert("DomainName", domain.to_string());
    params.insert("RRKeyWord", sub_domain.to_string());
    params.insert("TypeKeyWord", "TXT".to_string());
    complete_param(access_key_secret, params)
}

fn fill_delete_param(
    access_key_secret: &str,
    params: &mut BTreeMap<&'static str, String>,
    record_id: &str,
) {
    params.insert("Action", "DeleteDomainRecord".to_string());
    params.insert("RecordId", record_id.to_string());
    complete_param(access_key_secret, params)
}

fn gen_update_param<'a>(
    config: &AliyunConfig,
    domain: &str,
    sub_domain: &str,
    txt_value: &str,
) -> BTreeMap<&'static str, String> {
    let mut params = gen_basic_request_param(&config.access_key_id);
    fill_update_param(
        &config.access_key_secret,
        &mut params,
        domain,
        sub_domain,
        txt_value,
    );
    params
}

fn gen_desc_param<'a>(
    config: &AliyunConfig,
    domain: &str,
    sub_domain: &str,
) -> BTreeMap<&'static str, String> {
    let mut params = gen_basic_request_param(&config.access_key_id);
    fill_desc_param(&config.access_key_secret, &mut params, domain, sub_domain);
    params
}

fn gen_delete_param(config: &AliyunConfig, record_id: &str) -> BTreeMap<&'static str, String> {
    let mut params = gen_basic_request_param(&config.access_key_id);
    fill_delete_param(&config.access_key_secret, &mut params, record_id);
    params
}

async fn update<T: AsRef<str>>(config: &AliyunConfig, request: &DnsConfig, txt_value: T) -> Result<()> {
    info!(
        "update {}.{} TXT record to :{}",
        &request.domain, &request.sub_domain, txt_value.as_ref()
    );
    let client = Client::new();
    let params = gen_update_param(config, &request.domain, &request.sub_domain, txt_value.as_ref());
    let response = client
        .post(ALIYUN_API_URL)
        .form(&params)
        .send()
        .await?
        .text()
        .await?;

    info!("update Response: {}", response);
    Ok(())
}

pub async fn check<T: AsRef<str>>(request: &DnsConfig, txt_values: &[T]) -> Result<()> {
    // 创建默认的Resolver配置
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    // 发送TXT记录查询
    let domain = format!("{}.{}", request.sub_domain, request.domain);
    let delay = Duration::from_secs(10);
    // 略微高于20分钟应该就能保证生效
    for i in 0..120 {
        info!(
            "check records for domain: {} {}/60, this may take 10-minutes",
            &domain, i
        );
        let response = resolver
            .txt_lookup(&domain)
            .await
            .expect("Failed to lookup TXT record");
        let pass = txt_values.iter().all(|expected| {
            response.iter().any(|item|item.to_string().eq(&expected.as_ref()))
        });
        if pass {
            info!("delay 10s to wait let's encrypt");
            sleep(delay).await;
            return Ok(());
        } else {
            sleep(delay).await;
        }
    }

    Err(anyhow!("DNS TXT record not take effect after 15 minutes"))
}

#[derive(Deserialize, Debug)]
struct DescribeDomainRecordsResponse {
    #[serde(rename = "DomainRecords")]
    records: TxtRecordData,
}

#[derive(Deserialize, Debug)]
struct TxtRecordData {
    #[serde(rename = "Record")]
    pub record_list: Vec<TxtRecord>,
}

#[derive(Deserialize, Debug)]
struct TxtRecord {
    #[serde(rename = "RecordId")]
    pub record_id: String,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "RR")]
    pub rr: String,
    #[serde(rename = "Type")]
    pub record_type: String,
}

pub async fn clear<T: AsRef<str> + PartialEq>(
    config: &AliyunConfig,
    request: &DnsConfig,
    remain: &[T],
) -> Result<()> {
    let client = Client::new();

    // Step 1: 查询现有的TXT记录（使用 DescribeDomainRecords）
    let describe_params = gen_desc_param(config, &request.domain, &request.sub_domain);
    let response = client
        .post(ALIYUN_API_URL)
        .form(&describe_params)
        .send()
        .await?
        .json::<Option<DescribeDomainRecordsResponse>>()
        .await?;
    info!("TXT records for domain: {:?}", response);
    if let None = response {
        return Ok(());
    }
    let response = response.unwrap();

    // Step 2: 仅保留匹配的记录
    for record in response.records.record_list.iter() {
        if !remain.iter().any(|item| item.as_ref().eq(record.value.as_str())) {
            let delete_params = gen_delete_param(config, &record.record_id);
            let del_response = client
                .post(ALIYUN_API_URL)
                .form(&delete_params)
                .send()
                .await?
                .text()
                .await?;
            info!("Deleted TXT record {}: {}", record.record_id, del_response);
        }
    }
    Ok(())
}

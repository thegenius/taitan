use std::collections::BTreeMap;
use std::time::Duration;
use chrono::{Utc};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha1::Sha1;
use urlencoding::encode;
use rand::Rng;
use base64::{Engine as _, engine::general_purpose};
use anyhow::{anyhow, Result};
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::{Resolver, TokioAsyncResolver};
use serde::Deserialize;
use tokio::time::sleep;
use tracing::{error, info};
use crate::update_dns_txt;

type HmacSha1 = Hmac<Sha1>;

const ALIYUN_API_URL: &str = "https://alidns.aliyuncs.com";

#[derive(Debug)]
pub struct AliyunConfig {
    pub(crate) access_key_id: String,
    pub(crate) access_key_secret: String,
}

#[derive(Debug)]
pub struct UpdateDnsTxtRequest {
    pub domain: String,
    pub sub_domain: String,
    pub txt_value: String,
}

impl UpdateDnsTxtRequest {
    pub fn new(domain: String, sub_domain: String) -> Self {
        Self {
            domain,
            sub_domain,
            txt_value: String::new(),
        }
    }
}


pub async fn update_dns_txt_record(config: &AliyunConfig, request: &UpdateDnsTxtRequest) -> Result<()> {
    if let Err(err) = update(&config, &request).await {
        error!("{}", err);
        return Err(anyhow!(err));
    };
    if let Err(err) = check(&request).await {
        error!("{}", err);
        return Err(anyhow!(err));
    }
    if let Err(err) = clear(&config, &request).await {
        error!("{}", err);
        return Err(anyhow!(err));
    }
    Ok(())
}


async fn update(config: &AliyunConfig, request: &UpdateDnsTxtRequest) -> Result<()> {
    info!("update {}.{} TXT record to :{}", &request.domain, &request.sub_domain, &request.txt_value);
    let client = Client::new();

    // 构建公共参数
    let mut params = BTreeMap::new();
    params.insert("Action", "AddDomainRecord");
    params.insert("DomainName", &request.domain);
    params.insert("RR", &request.sub_domain);
    params.insert("Type", "TXT");
    params.insert("Value", &request.txt_value);
    params.insert("Version", "2015-01-09");
    params.insert("Format", "JSON");
    params.insert("AccessKeyId", &config.access_key_id);
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    params.insert("Timestamp", &timestamp);
    params.insert("SignatureMethod", "HMAC-SHA1");
    params.insert("SignatureVersion", "1.0");
    let nonce =  &rand::thread_rng().gen_range(0..u64::MAX).to_string();
    params.insert("SignatureNonce", nonce);

    // 生成签名
    let signature = generate_signature(&params, &config.access_key_secret);
    params.insert("Signature", &signature);

    // 发送请求
    let response = client.post(ALIYUN_API_URL)
        .form(&params)
        .send()
        .await?
        .text().await?;


    info!("update Response: {}", response);
    Ok(())
}

fn generate_signature(params: &BTreeMap<&str, &str>, secret: &str) -> String {
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


pub async fn check(request: &UpdateDnsTxtRequest) -> Result<()> {
    // 创建默认的Resolver配置
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    // 发送TXT记录查询
    let domain = format!("{}.{}", request.sub_domain, request.domain);
    let delay = Duration::from_secs(10);
    // 略微高于10分钟应该就能保证生效
    for i in 0..100 {
        info!("check records for domain: {} {}/60, this may take 10-minutes", &domain, i);
        let response = resolver.txt_lookup(&domain).await.expect("Failed to lookup TXT record");
        for rdata in response.iter() {
            let val = rdata.to_string();
            info!("check records for domain: {} found: {}", &domain, val);
            if val.eq(&request.txt_value) {
                return Ok(());
            }
        }
        sleep(delay).await;
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

pub async fn clear(
    config: &AliyunConfig,
    request: &UpdateDnsTxtRequest,
) -> Result<()> {
    let client = Client::new();

    // Step 1: 查询现有的TXT记录（使用 DescribeDomainRecords）
    let mut describe_params = BTreeMap::new();
    describe_params.insert("Action", "DescribeDomainRecords");
    describe_params.insert("DomainName", &request.domain);
    describe_params.insert("RRKeyWord", &request.sub_domain);
    describe_params.insert("TypeKeyWord", "TXT");
    describe_params.insert("Version", "2015-01-09");
    describe_params.insert("Format", "JSON");
    describe_params.insert("AccessKeyId", &config.access_key_id);
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    describe_params.insert("Timestamp", &timestamp);
    describe_params.insert("SignatureMethod", "HMAC-SHA1");
    describe_params.insert("SignatureVersion", "1.0");
    let nonce = &rand::thread_rng().gen_range(0..u64::MAX).to_string();
    describe_params.insert("SignatureNonce", nonce);

    let signature = generate_signature(&describe_params, &config.access_key_secret);
    describe_params.insert("Signature", &signature);

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
    // Step 2: 查找匹配的记录
    for record in response.records.record_list.iter() {
        if record.value != request.txt_value {

            // Step 3: 删除不匹配的记录
            let mut delete_params = BTreeMap::new();
            delete_params.insert("Action", "DeleteDomainRecord");
            delete_params.insert("RecordId", &record.record_id);
            delete_params.insert("Version", "2015-01-09");
            delete_params.insert("Format", "JSON");
            delete_params.insert("AccessKeyId", &config.access_key_id);
            let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            delete_params.insert("Timestamp", &timestamp);
            delete_params.insert("SignatureMethod", "HMAC-SHA1");
            delete_params.insert("SignatureVersion", "1.0");
            let nonce = &rand::thread_rng().gen_range(0..u64::MAX).to_string();
            delete_params.insert("SignatureNonce", nonce);

            let signature = generate_signature(&delete_params, &config.access_key_secret);
            delete_params.insert("Signature", &signature);

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
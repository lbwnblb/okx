use std::io::Write;
use std::ops::Add;
use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use env_logger::Builder;
use hmac::{Hmac, Mac};
use log::{info, LevelFilter};
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use reqwest::header::HeaderMap;
use sha2::Sha256;
use crate::common::config::{OKX_SIMULATION_API_KEY, OKX_SIMULATION_SECRET_KEY, OK_SIMULATION_ACCESS_PASSPHRASE, REST_URL};
static HTTP_CLIENT:Lazy<Client> = Lazy::new(||{
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().unwrap());
        // 可以自定义配置，例如设置超时、User-Agent 等
        Client::builder()
            .timeout(std::time::Duration::from_secs(10))  // 10 秒超时
            .build()
            .expect("Failed to build HTTP client")
});


pub fn get_client()->&'static Client{
    &HTTP_CLIENT
}

pub struct HttpClient;
impl HttpClient {
    pub async fn get(path: &str, params: Option<&[(&str, &str)]>)-> Result<Response, reqwest::Error>{
        let client = get_client();
        let url = format!("{REST_URL}{path}");
        let request_builder = client.get(url.as_str());
        match params {
            None => {
                Ok(request_builder.send().await?)
            }
            Some(params) => {
                Ok(request_builder.query(params).send().await?)
            }
        }
    }
}
pub struct HttpClientSimulation;
impl HttpClientSimulation {
    pub async fn get(path: &str, params: Option<&[(&str, &str)]>)-> Result<Response, reqwest::Error>{
        let now_iso = utc_now_iso();
        let client = get_client();
        let url = format!("{REST_URL}{path}");
        let request_builder = client.get(url.as_str());
        let mut request_builder = request_builder.header("OK-ACCESS-KEY", OKX_SIMULATION_API_KEY.as_str())
            .header("OK-ACCESS-TIMESTAMP", &now_iso).header("OK-ACCESS-PASSPHRASE", OK_SIMULATION_ACCESS_PASSPHRASE.as_str());
        match params {
            None => {
                let sign = sign(&now_iso, "GET", path,OKX_SIMULATION_SECRET_KEY.as_str());
                request_builder = request_builder.header("OK-ACCESS-SIGN",sign);
                Ok(request_builder.send().await?)
            }
            Some(params) => {
                let path_and_query = request_builder.try_clone().unwrap().query(params).build()?;
                let query = path_and_query.url().query().unwrap();
                let path_and_query = format!("{}?{}", path,query);
                let sign = sign(&now_iso, "GET", path_and_query.as_ref(),OKX_SIMULATION_SECRET_KEY.as_str());
                request_builder = request_builder.header("OK-ACCESS-SIGN",sign);
                Ok(request_builder.send().await?)
            }
        }
    }
}

pub fn log_init(){
    let offset = FixedOffset::east_opt(8 * 3600).unwrap();  // 定义 UTC+8 偏移变量

    Builder::new()
        .format(move |buf, record| {  // move 闭包捕获 offset
            let utc_now = Utc::now();  // 每次日志时获取当前 UTC 时间
            let local_now = offset.from_utc_datetime(&utc_now.naive_utc());  // 应用 +8 偏移
            writeln!(
                buf,
                "[{}] {}: {}",
                local_now.format("%Y-%m-%d %H:%M:%S%.3f"),  // 格式化：2025-11-16 15:23:00.123 (UTC+8)
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

}

pub fn price_to_tick_int_str(price:&str, tick_size:&str)->u64{
    if !price.contains(".") {
        info!("price_to_tick_int_str: price is not decimal");
        return u64::from_str(price).unwrap();
    }
    let price_split: Vec<&str> = price.split('.').collect();

    let tick_size_split= tick_size.split(".").collect::<Vec<&str>>();
    //小于1处理
    if price_split[0].eq("0")  {
        let tick_size_split_len = tick_size_split[1].len();
        let price_split_len = price_split[1].len();
        if price_split_len < tick_size_split_len {
            let price_split_1 = price_split[1].to_string();
            let price_split_1 = price_split_1.add("0");
            let result = u64::from_str(&price_split_1).unwrap();
            return result;
        }
        let price_split_1 = price_split[1].to_string();
        let result = u64::from_str(&price_split_1).unwrap();
        return  result
    }

    let tick_size_split_len = tick_size_split[1].len();
    let price_split_len = price_split[1].len();
    let price_split_1 = price_split[1].to_string();
    let price_split_0 = price_split[0].to_string();
    if price_split_len < tick_size_split_len {
        let price_split_1 = price_split_1.add("0");
        let result = u64::from_str(&price_split_0.add(price_split_1.as_ref())).unwrap();
        return result;
    }
    let result = u64::from_str(&price_split_0.add(price_split_1.as_ref())).unwrap();
    result

}
/// 返回类似 "2020-12-08T09:08:57.715Z" 的 UTC 时间字符串
pub fn utc_now_iso() -> String {
    // 获取当前 UTC 时间，精度到毫秒
    let now: DateTime<Utc> = Utc::now();
    // 格式化：固定使用 3 位毫秒 + Z
    now.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}
pub fn sign(timestamp:&str, method:&str, path:&str,secret_key:&str)->String{
    let params = format!("{}{}{}", timestamp, method, path);
    // 创建 HMAC-SHA256 实例
    let mut mac = Hmac::<Sha256>::new_from_slice(params.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(secret_key.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    BASE64_STANDARD.encode(code_bytes)
}

#[cfg(test)]
mod test{
    #[tokio::test]
    async fn test_price_to_tick_int_str(){
        let price = "0.1";
        let tick_size = "0.1";
        let result = super::price_to_tick_int_str(price, tick_size);
        println!("result: {}", result);
    }
}
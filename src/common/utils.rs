use std::io::Write;
use chrono::{FixedOffset, TimeZone, Utc};
use env_logger::Builder;
use log::LevelFilter;
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use reqwest::header::HeaderMap;
use crate::common::config::REST_URL;

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


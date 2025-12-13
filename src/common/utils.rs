use crate::common::config::{
    OK_SIMULATION_ACCESS_PASSPHRASE, OKX_SIMULATION_API_KEY, OKX_SIMULATION_SECRET_KEY,
    REST_SIMULATION_URL, REST_URL,
};
use crate::common::rest_api::SwapInstrument;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use env_logger::Builder;
use hmac::{Hmac, Mac};
use log::{LevelFilter, info};
use once_cell::sync::Lazy;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use sha2::Sha256;
use sonic_rs::{JsonValueTrait, Value, json, to_string};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Write};
use std::ops::Add;
use std::path::Path;
use std::str::FromStr;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message::Text;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use uuid::Uuid;

static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    // 可以自定义配置，例如设置超时、User-Agent 等
    Client::builder()
        .timeout(std::time::Duration::from_secs(10)) // 10 秒超时
        .build()
        .expect("Failed to build HTTP client")
});

pub static INSTRUMENTS_MAP: Lazy<HashMap<String, SwapInstrument>> = Lazy::new(|| {
    let vec = sonic_rs::from_reader::<BufReader<File>, Vec<SwapInstrument>>(BufReader::new(
        File::open("data/instruments.json").unwrap(),
    ))
    .unwrap();
    vec.into_iter()
        .filter(|instrument| instrument.settle_ccy.eq("USDT"))
        .map(|instrument| (instrument.inst_id.clone(), instrument))
        .collect::<HashMap<String, SwapInstrument>>()
});

pub fn get_sz(inst_id: &str) -> Option<&String> {
    if let Some(instrument) = get_swap_instrument(inst_id) {
        return Some(&instrument.tick_sz);
    };
    None
}
pub fn get_min_sz(inst_id:&str) ->Option<&String>{
    if let Some(instrument) = get_swap_instrument(inst_id) {
        return Some(&instrument.min_sz);
    };
    None

}
pub fn get_swap_instrument(inst_id: &str) -> Option<&SwapInstrument> {
    INSTRUMENTS_MAP.get(inst_id)
}

pub fn get_quantity_sz(inst_id: &str,quantity:&str)->String{
    let min_sz = get_min_sz(inst_id).unwrap();
    let quantity_to = price_to_tick_int_str(quantity, min_sz);
    let ct_val = get_ct_val(inst_id);
    let to_tick_int_str = price_to_tick_int_str("1", ct_val);
    
    tick_int_to_price_str(quantity_to*to_tick_int_str,get_min_sz(inst_id).unwrap())
}
#[cfg(test)]
mod utils_test {
    use super::*;
    #[test]
    fn test_get_client() {
        println!("{}", INSTRUMENTS_MAP.len());
        // for key in INSTRUMENTS_MAP.keys() {
        //     println!("{}", key);
        // }
    }
}

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}

pub struct HttpClient;
impl HttpClient {
    pub async fn get(
        path: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<Response, reqwest::Error> {
        let client = get_client();
        let url = format!("{REST_URL}{path}");
        let request_builder = client.get(url.as_str());
        match params {
            None => Ok(request_builder.send().await?),
            Some(params) => Ok(request_builder.query(params).send().await?),
        }
    }
}
pub struct HttpClientSimulation;
impl HttpClientSimulation {
    pub async fn get(
        path: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<Response, reqwest::Error> {
        let now_iso = utc_now_iso();
        let client = get_client();
        let url = format!("{REST_SIMULATION_URL}{path}");
        let request_builder = client.get(url.as_str());
        let mut request_builder = request_builder
            .header("OK-ACCESS-KEY", OKX_SIMULATION_API_KEY.as_str())
            .header("OK-ACCESS-TIMESTAMP", &now_iso)
            .header(
                "OK-ACCESS-PASSPHRASE",
                OK_SIMULATION_ACCESS_PASSPHRASE.as_str(),
            )
            .header("x-simulated-trading", "1");
        match params {
            None => {
                let sign = sign(
                    &now_iso,
                    "GET",
                    path,
                    "",
                    OKX_SIMULATION_SECRET_KEY.as_str(),
                );
                request_builder = request_builder.header("OK-ACCESS-SIGN", sign);
                Ok(request_builder.send().await?)
            }
            Some(params) => {
                request_builder = request_builder.query(params);
                let path_and_query = request_builder.try_clone().unwrap().build()?;
                let query = path_and_query.url().query().unwrap();
                let path_and_query = format!("{}?{}", path, query);
                let sign = sign(
                    &now_iso,
                    "GET",
                    path_and_query.as_ref(),
                    "",
                    OKX_SIMULATION_SECRET_KEY.as_str(),
                );
                request_builder = request_builder.header("OK-ACCESS-SIGN", sign);
                Ok(request_builder.send().await?)
            }
        }
    }
    pub async fn post(path: &str, json: Value) -> Result<Response, reqwest::Error> {
        let now_iso = utc_now_iso();
        let client = get_client();
        let url = format!("{REST_SIMULATION_URL}{path}");
        let request_builder = client.post(url.as_str());
        let mut request_builder = request_builder
            .header("OK-ACCESS-KEY", OKX_SIMULATION_API_KEY.as_str())
            .header("OK-ACCESS-TIMESTAMP", &now_iso)
            .header(
                "OK-ACCESS-PASSPHRASE",
                OK_SIMULATION_ACCESS_PASSPHRASE.as_str(),
            )
            .header("x-simulated-trading", "1");
        request_builder = request_builder.json(&json);
        let sign = sign(
            &now_iso,
            "POST",
            path,
            to_string(&json).unwrap().as_str(),
            OKX_SIMULATION_SECRET_KEY.as_str(),
        );
        request_builder = request_builder.header("OK-ACCESS-SIGN", sign);
        Ok(request_builder.send().await?)
    }
}

pub fn send_str(value: &str) -> Message {
    Text(Utf8Bytes::from(value))
}

pub fn get_ct_val(inst_id: &str)->&str{
    &get_swap_instrument(inst_id).unwrap().ct_val
}

pub fn log_init() {
    let offset = FixedOffset::east_opt(8 * 3600).unwrap(); // 定义 UTC+8 偏移变量

    Builder::new()
        .format(move |buf, record| {
            // move 闭包捕获 offset
            let utc_now = Utc::now(); // 每次日志时获取当前 UTC 时间
            let local_now = offset.from_utc_datetime(&utc_now.naive_utc()); // 应用 +8 偏移
            writeln!(
                buf,
                "[{}] {}: {}",
                local_now.format("%Y-%m-%d %H:%M:%S%.3f"), // 格式化：2025-11-16 15:23:00.123 (UTC+8)
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}

pub fn price_to_tick_int_str(price: &str, tick_size: &str) -> u64 {
    if !price.contains(".") && !tick_size.contains(".") {
        info!("price_to_tick_int_str: price is not decimal");
        return u64::from_str(price).unwrap();
    }
    if !price.contains(".") && tick_size.contains(".") {
        let tick_size_sp: Vec<&str> = tick_size.split(".").collect();
        let v = format!("{}{}", price, tick_size_sp[1].replace("1", "0"));
        return u64::from_str(v.as_str()).unwrap()
    }

    let price_split: Vec<&str> = price.split('.').collect();

    let tick_size_split = tick_size.split(".").collect::<Vec<&str>>();
    //小于0处理
    if price_split[0].eq("0") {
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
        return result;
    }
    //大于0处理
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

pub fn tick_int_to_price_str(tick_int: u64, tick_size: &str) -> String {
    // 如果 tick_size 没有小数点，直接返回整数字符串
    if !tick_size.contains(".") {
        return tick_int.to_string();
    }

    let tick_size_split: Vec<&str> = tick_size.split('.').collect();
    let decimal_places = tick_size_split[1].len();
    let tick_int_str = tick_int.to_string();
    let tick_int_len = tick_int_str.len();

    // 如果整数长度小于等于小数位数，说明整数部分为 0
    if tick_int_len <= decimal_places {
        // 补充前导 0
        let padding = "0".repeat(decimal_places - tick_int_len);
        return format!("0.{}{}", padding, tick_int_str);
    }

    // 整数长度大于小数位数，需要分割
    let split_pos = tick_int_len - decimal_places;
    let integer_part = &tick_int_str[..split_pos];
    let decimal_part = &tick_int_str[split_pos..];

    format!("{}.{}", integer_part, decimal_part)
}

#[cfg(test)]
mod test_p {
    use super::*;
    #[test]
    fn test_price_to_tick_int_str() {
        let inst_id = "BTC-USDT-SWAP";
        let min_sz = get_min_sz(inst_id).unwrap();
        let int_str = price_to_tick_int_str("0", min_sz);
        println!("{}", int_str);
    }
    #[test]
    fn test_tick_int_to_price_str() {
        let inst_id = "BTC-USDT-SWAP";
        let min_sz = get_min_sz(inst_id).unwrap();
        let price_str = tick_int_to_price_str(7371, min_sz);
        println!("{}", price_str);
    }
}

/// 返回类似 "2020-12-08T09:08:57.715Z" 的 UTC 时间字符串
pub fn utc_now_iso() -> String {
    // 获取当前 UTC 时间，精度到毫秒
    let now: DateTime<Utc> = Utc::now();
    // 格式化：固定使用 3 位毫秒 + Z
    now.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}
pub fn sign(timestamp: &str, method: &str, path: &str, body: &str, secret_key: &str) -> String {
    // 拼接：timestamp + method + requestPath + body
    let message = format!("{}{}{}{}", timestamp, method, path, body);

    // 用 secret_key 作为密钥创建 HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    // 用拼接的消息计算签名
    mac.update(message.as_bytes());

    let result = mac.finalize();
    BASE64_STANDARD.encode(result.into_bytes())
}


pub fn get_inst_id_code(inst_id: &str) -> String {
    get_swap_instrument(inst_id).unwrap().inst_id_code.to_string()
}

pub fn order_id_str(inst_id: &str,side:&str,price:&str,order_type:&str)-> String {
    format!("{}-{}-{}-{}",inst_id,side,price,order_type)
}

pub const WS_FILE_PATH: &str = "data/input.txt";
pub fn read_ws_file() -> Lines<BufReader<File>> {
    let path = Path::new(WS_FILE_PATH);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    reader.lines()
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn read_test() {
        log_init();
        let path = Path::new(WS_FILE_PATH);
        
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();  // 处理可能的 IO 错误
            let okx_msg:OkxMessage = from_str(line.as_str()).unwrap();
            println!("{:?}",okx_msg)
        }
    }
    use std::io::BufRead;
use std::fs::{read, write};
    use std::path::Path;
    use sonic_rs::{from_reader, from_str};
    use tokio::fs::read_link;
    use tokio::io::AsyncBufReadExt;
    use super::*;
    use crate::common::rest_api::ticker;
    use crate::common::ws_api::{BookData, Books, OkxMessage};

    #[tokio::test]
    async fn test_okx_simulation_api_account_balance() {
        // 测试模拟盘账户余额查询（需要签名的私有接口）
        let result = HttpClientSimulation::get("/api/v5/account/balance?ccy=BTC", None).await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap();
                println!("Status: {}", status);
                println!("Response: {}", body);
                assert!(status.is_success(), "API 请求失败: {}", body);
            }
            Err(e) => {
                panic!("请求失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_okx_simulation_api_positions() {
        // 测试模拟盘持仓查询
        let result = HttpClientSimulation::get("/api/v5/account/positions", None).await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap();
                println!("Status: {}", status);
                println!("持仓信息: {}", body);
                assert!(status.is_success(), "持仓查询失败: {}", body);
            }
            Err(e) => {
                panic!("请求失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_okx_simulation_api_with_params() {
        // 测试带查询参数的 API（查询特定币种余额）
        let params = &[("ccy", "BTC")];
        let result = HttpClientSimulation::get("/api/v5/account/balance", Some(params)).await;

        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap();
                println!("Status: {}", status);
                println!("USDT 余额: {}", body);
                assert!(status.is_success(), "查询失败: {}", body);
            }
            Err(e) => {
                panic!("请求失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_price_to_tick_int_str() {
        let inst_id = "BTC-USDT-SWAP";

        // println!("result: {}", result);

        let mut map_book_vec:HashMap<(String, u64, u64),Vec<u64>> = HashMap::new();
        let book_json_vec = from_reader::<BufReader<File>, Vec<BookData>>(BufReader::new(File::open("data/books.json").unwrap())).unwrap();
        let book_data = book_json_vec.into_iter().last().unwrap();
        let vec_asks = book_data.asks.into_iter().map(|vec_str| (price_to_tick_int_str(vec_str.get(0).unwrap(), get_sz(inst_id).unwrap()),price_to_tick_int_str(vec_str.get(1).unwrap(), get_min_sz(inst_id).unwrap()))).collect::<Vec<(u64,u64)>>();
        // let vec_bids = book_data.bids;
        let max_price = vec_asks.iter().map(|(price, _)| { price }).max().unwrap();
        let min_price = vec_asks.iter().map(|(price, _)| { price }).min().unwrap();
        let sub_price = max_price - min_price;
        println!("max_price: {}, min_price: {}, sub_price: {}", max_price, min_price, sub_price);
        let mut vec_price_v = vec![0u64;(sub_price+1) as usize];
        println!("{}", vec_price_v.len());
        println!("{}",max_price-min_price);
        vec_asks.iter().for_each(|(price,sz)| {
            println!("index: {},price:{} min_price:{} sz: {}", price-min_price, price,min_price, sz);
            vec_price_v[(price-min_price) as usize] = *sz;
        });

        map_book_vec.insert(("BTC-USDT-SWAP".to_string(),min_price.clone(),max_price.clone()),vec_price_v);
        
    }
    #[tokio::test]
    async fn order_test() {
        let response = HttpClientSimulation::post(
            "/api/v5/trade/order",
            json!({
              "instId": "ETH-USDT",
              "tdMode": "cash",
              "side": "sell",
              "ordType": "limit",
              "px": "4000",
              "sz": "0.01"
            }),
        )
        .await
        .unwrap();
        println!("{}", response.text().await.unwrap());
    }

}

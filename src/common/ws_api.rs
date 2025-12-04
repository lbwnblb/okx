use crate::common::config::*;
use log::info;
use sonic_rs::{json, Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use crate::common::utils::sign;

pub async fn create_ws(url: &str) ->Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>{
    // 创建连接并获取 WebSocket 流
    let (ws_stream, response) = connect_async(url).await?;
    // Print the response status
    info!("ws_url:{} Response status: {}",url,response.status().as_str());
    Ok(ws_stream)
}

#[derive(Debug, Deserialize)]
pub struct OkxMessage {
    pub event: Option<String>,
    pub arg: Option<Arg>,
}

#[derive(Debug, Deserialize)]
pub struct Arg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}
#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub data: Vec<TickerData>,
}
#[derive(Debug, Deserialize)]
pub struct Books{
    /// 消息的动作类型，例如 "snapshot" 或 "update"。
    #[serde(rename = "action")]
    pub action: String,
    /// 包含订单簿数据的数组。
    #[serde(rename = "data")]
    pub data: Vec<BookData>,
}
/// BookData 结构体包含实际的订单簿快照数据。
#[derive(Debug, Serialize, Deserialize)]
pub struct BookData {
    /// 卖方报价列表（asks）。
    #[serde(rename = "asks")]
    pub asks: Vec<Vec<String>>,
    /// 买方报价列表（bids）。
    #[serde(rename = "bids")]
    pub bids: Vec<Vec<String>>,
    /// 数据快照的时间戳（毫秒字符串）。
    #[serde(rename = "ts")]
    pub ts: String,
    /// 用于数据验证的校验和。
    #[serde(rename = "checksum")]
    pub checksum: i64,
    /// 上一个序列号。-1 通常表示快照。
    #[serde(rename = "prevSeqId")]
    pub prev_seq_id: i64,
    /// 序列号，用于跟踪更新。
    #[serde(rename = "seqId")]
    pub seq_id: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")] // 自动把 JSON 的驼峰(lastSz)转为 Rust 的蛇形(last_sz)
pub struct TickerData {
    pub inst_type: String,
    pub inst_id: String,

    // 注意：OKX 为了精度，价格和数量都返回 String，不要直接用 f64
    pub last: String,       // 最新成交价
    pub last_sz: String,    // 最新成交量

    pub ask_px: String,     // 卖一价
    pub ask_sz: String,     // 卖一量

    pub bid_px: String,     // 买一价
    pub bid_sz: String,     // 买一量

    pub open24h: String,    // 24小时开盘价
    pub high24h: String,    // 24小时最高价
    pub low24h: String,     // 24小时最低价

    #[serde(rename = "sodUtc0")]
    pub sod_utc0: String,   // UTC 0点开盘价
    #[serde(rename = "sodUtc8")]
    pub sod_utc8: String,   // UTC+8 开盘价

    pub vol_ccy24h: String, // 24小时成交量（币）
    pub vol24h: String,     // 24小时成交量（张/USDT）

    pub ts: String,         // 时间戳
}

pub const CHANNEL_TICKERS: &str = "tickers";
pub const CHANNEL_BOOKS: &str = "books";
pub const CHANNEL_BBO_TBT: &str = "bbo-tbt";

pub fn subscribe(channel: &str,inst_id: &str)->String{
    json!( {
        "op": "subscribe",
        "args": [{
            "channel": channel,
            "instId": inst_id
        }]
    }).to_string()
}

pub fn login()->String{
    let timestamp = OffsetDateTime::now_utc().unix_timestamp();

    let sign  = sign(timestamp.to_string().as_str(), "GET", "/users/self/verify", "",get_secret_key());
        json!({
 "op": "login",
 "args":
  [
     {
       "apiKey": get_api_key(),
       "passphrase": get_passphrase(),
       "timestamp": timestamp,
       "sign":sign
      }
   ]
}).to_string()
}
pub fn order()->String{
    json!({
    "id": "1512",
    "op": "order",
    "args": [{
        "side": "buy",
        "instId": "BTC-USDT",
        "tdMode": "isolated",
        "ordType": "market",
        "sz": "0.1"
    }]
}
).to_string()
}

#[cfg(test)]
mod ws_test{
    use crate::common::config::OK_SIMULATION_ACCESS_PASSPHRASE;
    use crate::common::config::OKX_SIMULATION_SECRET_KEY;
    use crate::common::config::OKX_SIMULATION_API_KEY;
    use futures::{SinkExt, StreamExt};
    use sonic_rs::{json, to_string};
    use time::OffsetDateTime;
    use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
    use crate::common::config::WS_SIMULATION_URL_PRIVATE;
    use crate::common::utils::sign;
    use crate::common::ws_api::create_ws;

    #[tokio::test]
    async fn test_login(){
        // create_ws();
        let web_socket_stream = create_ws(WS_SIMULATION_URL_PRIVATE).await.unwrap();
        let (mut tx, mut rx) = web_socket_stream.split();
        let timestamp = OffsetDateTime::now_utc().unix_timestamp();
        let sign  = sign(timestamp.to_string().as_str(), "GET", "/users/self/verify", "",OKX_SIMULATION_SECRET_KEY.as_str());
        let x = to_string(&json!({
 "op": "login",
 "args":
  [
     {
       "apiKey": OKX_SIMULATION_API_KEY.as_str(),
       "passphrase": OK_SIMULATION_ACCESS_PASSPHRASE.as_str(),
       "timestamp": timestamp,
       "sign":sign
      }
   ]
}
)).unwrap();
        tx.send(Message::Text(Utf8Bytes::from(x))).await.unwrap();
        loop {
            let option = rx.next().await;
            match option {
                Some(Ok(msg)) => {
                    println!("Received message: {:?}", msg);
                }
                Some(Err(e)) => {
                    println!("Error: {:?}", e);
                }
                None => {
                    println!("WebSocket connection closed.");
                    break;
                }
            }
        }
    }
}
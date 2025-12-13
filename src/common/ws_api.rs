use crate::common::config::*;
use log::info;
use sonic_rs::{json, Deserialize, Serialize};
use time::OffsetDateTime;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::frame::coding::Data;
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

/// Books5 结构体（只包含前5档）
#[derive(Debug, Deserialize)]
pub struct Books5 {
    #[serde(rename = "action")]
    pub action: Option<String>,
    #[serde(rename = "data")]
    pub data: Vec<Book5Data>,
    // OKX 错误响应可能包含 code (integer 类型)
    #[serde(default, deserialize_with = "deserialize_code_as_string")]
    pub code: Option<String>,
    #[serde(rename = "msg")]
    pub msg: Option<String>,
}

// 自定义反序列化函数：接受 integer 或 string 并统一转为 Option<String>
fn deserialize_code_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    use sonic_rs::{Value, JsonValueTrait};
    
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(v) => {
            if v.is_str() {
                Ok(Some(v.as_str().unwrap().to_string()))
            } else if v.is_i64() {
                Ok(Some(v.as_i64().unwrap().to_string()))
            } else if v.is_u64() {
                Ok(Some(v.as_u64().unwrap().to_string()))
            } else if v.is_f64() {
                Ok(Some(v.as_f64().unwrap().to_string()))
            } else {
                Ok(None)
            }
        }
    }
}

/// Book5Data 结构体 - books5 不包含 checksum 等字段
#[derive(Debug, Deserialize)]
pub struct Book5Data {
    #[serde(rename = "asks")]
    pub asks: Vec<Vec<String>>,
    #[serde(rename = "bids")]
    pub bids: Vec<Vec<String>>,
    #[serde(rename = "instId")]
    pub inst_id: String,
    #[serde(rename = "ts")]
    pub ts: String,
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

#[derive(Debug, Deserialize)]
pub struct ChannelBboTbt {
    pub inst_id: String,
    pub data: Vec<BboTbtData>
}
#[derive(Debug, Deserialize)]
pub struct BboTbtData{
    pub asks: Vec<Vec<String>>,
    pub bids: Vec<Vec<String>>,
}

pub const CHANNEL_TICKERS: &str = "tickers";
pub const CHANNEL_BOOKS: &str = "books";
pub const CHANNEL_BOOKS5: &str = "books5";
pub const CHANNEL_BBO_TBT: &str = "bbo-tbt";

pub fn subscribe(channel: &str,inst_id: &str)->String{
    json!({
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
// pub fn order()->String{
//     json!({
//     "id": "1512",
//     "op": "order",
//     "args": [{
//         "side": "buy",
//         "instId": "BTC-USDT",
//         "tdMode": "cash",  // 改为 cash 模式（现货）
//         "ordType": "limit",  // 限价单更安全
//         "px": "50000",  // 添加价格
//         "sz": "0.0001",  // 减小数量避免资金不足
//         "ccy": "USDT"  // 关键：添加交易币种
//     }]
// }
// ).to_string()
// }

pub fn order(id: &str, side: &str, inst_id: &str, td_mode: &str, ord_type: &str, px: Option<&str>, sz: &str) ->String{
    let mut arg = json!({
        "side": side,
        "instId": inst_id,
        "tdMode": td_mode,
        "ordType": ord_type,
        "sz": sz,
        "posSide": "net"
    });

    if let Some(price) = px {
        arg["px"] = json!(price);
    }

    json!({
        "id": id,
        "op": "order",
        "args": [arg]
    }).to_string()
}

pub struct TdMode;
impl TdMode {
    //逐仓
    pub const ISOLATED: &'static str = "isolated";
    // 全仓
    pub const CROSS: &'static str = "cross";
}
pub struct  OrderType;
impl OrderType {
    pub const LIMIT: &'static str = "limit";
    pub const MARKET: &'static str = "market";
}
pub struct Side;
impl Side {
    pub const BUY: &'static str = "buy";
    pub const SELL: &'static str = "sell";
}
pub fn order_market(id: &str, side: &str, inst_id: &str,sz: &str)->String{
    let pos_side = if side == Side::BUY { "long" } else { "short" };
    order_market_with_pos(id, side, inst_id, sz, pos_side)
}

pub fn order_market_with_pos(id: &str, side: &str, inst_id: &str, sz: &str, pos_side: &str)->String{
    let arg = json!({
        "side": side,
        "instId": inst_id,
        "tdMode": TdMode::CROSS,
        "ordType": OrderType::MARKET,
        "sz": sz,
        "posSide": pos_side
    });

    json!({
        "id": id,
        "op": "order",
        "args": [arg]
    }).to_string()
}




pub fn order_buy(id: &str,inst_id: &str){

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
    use crate::common::utils::{send_str, sign};
    use crate::common::ws_api::{create_ws, order};

//     #[tokio::test]
//     async fn test_login(){
//         // create_ws();
//         let web_socket_stream = create_ws(WS_SIMULATION_URL_PRIVATE).await.unwrap();
//         let (mut tx, mut rx) = web_socket_stream.split();
//         let timestamp = OffsetDateTime::now_utc().unix_timestamp();
//         let sign  = sign(timestamp.to_string().as_str(), "GET", "/users/self/verify", "",OKX_SIMULATION_SECRET_KEY.as_str());
//         let x = to_string(&json!({
//  "op": "login",
//  "args":
//   [
//      {
//        "apiKey": OKX_SIMULATION_API_KEY.as_str(),
//        "passphrase": OK_SIMULATION_ACCESS_PASSPHRASE.as_str(),
//        "timestamp": timestamp,
//        "sign":sign
//       }
//    ]
// }
// )).unwrap();
// let mut logged_in = false;
// let mut order_sent = false;
//         tx.send(Message::Text(Utf8Bytes::from(x))).await.unwrap();
//         loop {
//             let option = rx.next().await;
//             match option {
//                 Some(Ok(text)) => {
//                     let msg_str = text.to_string();
//
//                 // 登录成功
//                 if !logged_in && msg_str.contains("\"event\":\"login\"") && msg_str.contains("\"code\":\"0\"") {
//                     println!("✅ Login successful");
//                     logged_in = true;
//                     println!("Sending order...");
//                     tx.send(send_str(order().as_str())).await.unwrap();
//                     order_sent = true;
//                 }
//                 // 下单响应
//                 else if order_sent && msg_str.contains("\"op\":\"order\"") {
//                     println!("📦 Order response: {}", msg_str);
//                     break;
//                 }
//                 }
//                 Some(Err(e)) => {
//                     println!("Error: {:?}", e);
//                 }
//                 None => {
//                     println!("WebSocket connection closed.");
//                     break;
//                 }
//             }
//         }
//     }
}
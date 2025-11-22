use sonic_rs::{from_str, Deserialize, Serialize};
use crate::common::utils::{HttpClient};

// 主响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkxSwapInstrumentsResponse {
    pub code: String,
    pub msg: String,
    pub data: Vec<SwapInstrument>,
}

// 单个永续合约仪器结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInstrument {
    #[serde(alias = "alias")]
    pub alias: String,                          // 别名，通常为空字符串

    #[serde(alias = "auctionEndTime")]
    pub auction_end_time: String,               // 拍卖结束时间，通常为空

    #[serde(alias = "baseCcy")]
    pub base_ccy: Option<String>,               // 基础币种，币本位合约为空

    #[serde(alias = "category")]
    pub category: String,                       // 分类 "1" 表示主流币

    #[serde(alias = "contTdSwTime")]
    pub cont_td_sw_time: String,

    #[serde(alias = "ctMult")]
    pub ct_mult: String,                        // 合约乘数，通常为 "1"

    #[serde(alias = "ctType")]
    pub ct_type: String,                        // "linear 或 inverse

    #[serde(alias = "ctVal")]
    pub ct_val: String,                         // 合约面值（字符串，因为可能是小数）

    #[serde(alias = "ctValCcy")]
    pub ct_val_ccy: String,                     // 合约面值币种，如 USD、BTC、ETH

    #[serde(alias = "expTime")]
    pub exp_time: String,                       // 到期时间，永续合约为空

    #[serde(alias = "futureSettlement")]
    pub future_settlement: bool,

    #[serde(alias = "groupId")]
    pub group_id: String,

    #[serde(alias = "instFamily")]
    pub inst_family: Option<String>,            // 如 BTC-USD、BTC-USDT 等，有些可能为空

    #[serde(alias = "instId")]
    pub inst_id: String,                        // 合约ID，如 BTC-USD-SWAP

    #[serde(alias = "instIdCode")]
    pub inst_id_code: i64,

    #[serde(alias = "instType")]
    pub inst_type: String,                      // SWAP

    #[serde(alias = "lever")]
    pub lever: String,                          // 最大杠杆，如 "100" "50"

    #[serde(alias = "listTime")]
    pub list_time: String,                      // 上线时间戳（毫秒）

    #[serde(alias = "lotSz")]
    pub lot_sz: String,                         // 下单张数精度，通常 "1"

    #[serde(alias = "maxIcebergSz")]
    pub max_iceberg_sz: String,

    #[serde(alias = "maxLmtAmt")]
    pub max_lmt_amt: String,

    #[serde(alias = "maxLmtSz")]
    pub max_lmt_sz: String,

    #[serde(alias = "maxMktAmt")]
    pub max_mkt_amt: Option<String>,

    #[serde(alias = "maxMktSz")]
    pub max_mkt_sz: String,

    #[serde(alias = "maxPlatOILmt")]
    pub max_plat_oi_lmt: Option<String>,

    #[serde(alias = "maxStopSz")]
    pub max_stop_sz: String,

    #[serde(alias = "maxTriggerSz")]
    pub max_trigger_sz: String,

    #[serde(alias = "maxTwapSz")]
    pub max_twap_sz: String,

    #[serde(alias = "minSz")]
    pub min_sz: String,                         // 最小下单张数

    #[serde(alias = "openType")]
    pub open_type: String,

    #[serde(alias = "optType")]
    pub opt_type: String,

    #[serde(alias = "posLmtAmt")]
    pub pos_lmt_amt: String,

    #[serde(alias = "posLmtPct")]
    pub pos_lmt_pct: String,

    #[serde(alias = "preMktSwTime")]
    pub pre_mkt_sw_time: String,

    #[serde(alias = "quoteCcy")]
    pub quote_ccy: Option<String>,

    #[serde(alias = "ruleType")]
    pub rule_type: String,

    #[serde(alias = "settleCcy")]
    pub settle_ccy: String,                     // 结算币种：BTC、USD、USDT、USDC 等

    #[serde(alias = "state")]
    pub state: String,                        // live / suspend 等

    #[serde(alias = "stk")]
    pub stk: String,

    #[serde(alias = "tickSz")]
    pub tick_sz: String,                        // 价格精度，如 "0.1" "0.01"

    #[serde(alias = "tradeQuoteCcyList")]
    pub trade_quote_ccy_list: Vec<String>,

    #[serde(alias = "uly")]
    pub uly: String,                            // 标的指数，如 BTC-USD、SOL-USDT
}



pub async fn instruments()->Result<OkxSwapInstrumentsResponse, Box<dyn std::error::Error>>{
    let result = HttpClient::get("/api/v5/public/instruments", Some(&[("instType", "SWAP")])).await;
    let result = result.unwrap().text().await?;
    let value = from_str::<OkxSwapInstrumentsResponse>(&result)?;
    // info!("{}", value);
    Ok(value)
}

/// 单个 ticker 数据项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticker {
    #[serde(rename = "instType")]
    pub inst_type: String,        // 产品类型：SWAP、SPOT 等

    #[serde(rename = "instId")]
    pub inst_id: String,          // 交易对，如 BTC-USDT-SWAP

    #[serde(rename = "last")]
    pub last: String,             // 最新成交价

    #[serde(rename = "lastSz")]
    pub last_sz: String,          // 最新成交数量

    #[serde(rename = "askPx")]
    pub ask_px: String,           // 卖一价

    #[serde(rename = "askSz")]
    pub ask_sz: String,           // 卖一量

    #[serde(rename = "bidPx")]
    pub bid_px: String,           // 买一价

    #[serde(rename = "bidSz")]
    pub bid_sz: String,           // 买一量

    #[serde(rename = "open24h")]
    pub open_24h: String,         // 24h 开盘价

    #[serde(rename = "high24h")]
    pub high_24h: String,         // 24h 最高价

    #[serde(rename = "low24h")]
    pub low_24h: String,          // 24h 最低价

    #[serde(rename = "volCcy24h")]
    pub vol_ccy_24h: String,      // 24h 交易量（计价币种）

    #[serde(rename = "vol24h")]
    pub vol_24h: String,          // 24h 交易量（币种数量）

    #[serde(rename = "ts")]
    pub ts: String,               // 数据产生时间（毫秒时间戳）

    #[serde(rename = "sodUtc0")]
    pub sod_utc0: String,         // UTC+0 周期开盘价

    #[serde(rename = "sodUtc8")]
    pub sod_utc8: String,         // UTC+8 周期开盘价
}

/// API 返回的最外层结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerResponse {
    pub code: String,
    pub msg: String,
    pub data: Vec<Ticker>,
}
pub async fn ticker(inst_id: &str)-> Result<TickerResponse, Box<dyn std::error::Error>>{
    let result = HttpClient::get("/api/v5/market/ticker", Some(&[("instId",inst_id)])).await;
    let result = result?.text().await;
    let result = from_str::<TickerResponse>(&result?)?;
    Ok(result)
}


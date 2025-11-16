use std::io::Write;
use chrono::{FixedOffset, TimeZone, Utc};
use env_logger::Builder;
use log::__private_api::enabled;
use log::LevelFilter;
use okx::contract::ws_api::create_ws;

#[tokio::main]
async fn main() ->Result<(), Box<dyn std::error::Error>>{
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
    create_ws().await?;
    Ok(())
}

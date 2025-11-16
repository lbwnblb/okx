use log::info;
use sonic_rs::{from_str, Value};
use crate::common::utils::{HttpClient};

pub async fn instruments()->Result<Value, Box<dyn std::error::Error>>{
    let result = HttpClient::get("/api/v5/public/instruments", Some(&[("instType", "SWAP")])).await;
    let result = result.unwrap().text().await?;
    let value = from_str::<Value>(&result)?;
    // info!("{}", value);
    Ok(value)
}
use log::info;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use crate::common::config::WS_URL;

pub async fn create_ws() ->Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>>{
    let request = WS_URL.into_client_request().unwrap();
    // 创建连接并获取 WebSocket 流
    let (ws_stream, response) = connect_async(request).await?;
    // Print the response status
    info!("ws_url:{} Response status: {}",WS_URL,response.status().as_str());
    Ok(ws_stream)
}
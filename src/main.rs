use std::error;
use std::sync::RwLock;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use sonic_rs::{from_str, JsonValueTrait};
use tokio::spawn;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver};
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio_tungstenite::tungstenite::Message::Text;
use tokio_tungstenite::tungstenite::{Error, Message, Utf8Bytes};
use okx::common::config::WS_SIMULATION_URL_PUBLIC;
use okx::common::rest_api::instruments;
use okx::common::utils::log_init;
use okx::common::ws_api::{create_ws, subscribe, Books, OkxMessage, Ticker, TickerData, CHANNEL_BOOKS, CHANNEL_TICKERS};

#[tokio::main]
async fn main() ->Result<(), Box<dyn error::Error>>{
    log_init();
    let ws = create_ws(WS_SIMULATION_URL_PUBLIC).await?;
    let (mut tx, mut rx) = ws.split();
    tx.send(Text(Utf8Bytes::from(subscribe(CHANNEL_TICKERS, "BTC-USDT-SWAP")))).await?;
    tx.send(Text(Utf8Bytes::from(subscribe(CHANNEL_BOOKS, "BTC-USDT-SWAP")))).await?;
    let (tx_ticker_data,rx_ticker_data) = channel::<TickerData>(512);
    let (tx_books_data,rx_books_data) = channel::<Books>(512);
    rx_ticker_data_spawn(rx_ticker_data);
    rx_books_data_spawn(rx_books_data);
    loop {
        let result = rx.next().await;
        match result {
            None => {
            }
            Some(result) => {
                match result {
                    Ok(message) => {
                        match message {
                            Text(text) => {
                                let okx_message = from_str::<OkxMessage>(&text)?;
                                if let Some(event) = okx_message.event{
                                        info!("event : {}", event);
                                        continue;
                                }
                                if let Some(arg) = okx_message.arg{
                                    let channel = arg.channel;
                                    match channel.as_str() {
                                        CHANNEL_TICKERS => {
                                            from_str::<Ticker>(&text).unwrap().data.into_iter().for_each(|item| {
                                                if let Err( err) = tx_ticker_data.try_send(item){
                                                    if let TrySendError::Full(_) = err {
                                                        error!("{} full",CHANNEL_TICKERS)
                                                    }
                                                };
                                            });
                                        },
                                        CHANNEL_BOOKS => {
                                            let books = from_str::<Books>(&text).unwrap();
                                            let result = tx_books_data.send(books).await;
                                            match result {
                                                Ok(_) => {},
                                                Err(err) => {
                                                    match err {
                                                        SendError(error) => {
                                                            error!("传输异常: {}",CHANNEL_BOOKS);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(error) => {
                        error!("{}", error)
                    }
                }
            }
        }
    }
}
pub  fn rx_ticker_data_spawn(mut rx: Receiver<TickerData>){
    spawn(async move {
        while let Some(item) = rx.recv().await {
            info!("{:?}",item);
        }
    });
}
pub  fn rx_books_data_spawn(mut rx: Receiver<Books>){
    let asks = Vec::<String>::new();
    spawn(async move {
        while let Some(item) = rx.recv().await {
            info!("{:?}",item);
        }
    });
}

#[cfg(test)]
mod test{
    use std::fs::File;
    use sonic_rs::{from_reader, to_writer_pretty};
    use sonic_rs::writer::BufferedWriter;
    use okx::common::rest_api::{instruments, ticker, SwapInstrument};

    #[tokio::test]
    async fn test_main(){
        let file_path = "data/instruments.json";

        let read_instruments = from_reader::<&mut File, Vec<SwapInstrument>>(&mut File::open(file_path).unwrap()).unwrap();
        for swap_instrument in read_instruments {
            let a = "83709.7";
            let swap_instrument_id = swap_instrument.inst_id;
            let tick_sz = swap_instrument.tick_sz;
            let split: Vec<&str> = a.split('.').collect();
            let first = split[0];   // 第一个值
            let second = split[1];  // 第二个值

            println!("{}  {}",swap_instrument_id,tick_sz);
        }
    }
    #[tokio::test]
    async fn ticker_test(){
        ticker("BTC-USDT-SWAP").await;
    }
}
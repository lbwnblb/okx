use std::collections::{BTreeMap, HashMap};
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
use okx::common::config::{get_ws_public};
use okx::common::rest_api::instruments;
use okx::common::utils::{log_init, send_str};
use okx::common::ws_api::{create_ws, login, order, subscribe, Books, OkxMessage, Ticker, TickerData, CHANNEL_BOOKS, CHANNEL_TICKERS};

#[tokio::main]
async fn main() ->Result<(), Box<dyn error::Error>>{
    log_init();
    let ws = create_ws(get_ws_public()).await?;
    let (mut tx, mut rx) = ws.split();
    //登录
    tx.send(send_str(subscribe(CHANNEL_BOOKS, "BTC-USDT-SWAP").as_str())).await?;
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
                                let result = from_str::<OkxMessage>(&text);
                                if let Ok (result) = result{
                                    if let Some(event) = result.event {
                                        info!("event {}", event);
                                        continue;
                                    }
                                    if let Some(args) = result.arg {
                                        match args.channel.as_str() {
                                            CHANNEL_BOOKS => {
                                                let books = from_str::<Books>(&text).unwrap();
                                                let data_vec = books.data;
                                                for boo_data in data_vec {
                                                    for bid in boo_data.bids {
                                                        let x_0 = bid.get(0).unwrap();
                                                        info!("价格:{}", x_0);
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
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
    Ok(())
}
pub  fn rx_ticker_data_spawn(mut rx: Receiver<TickerData>){
    spawn(async move {
        while let Some(item) = rx.recv().await {
            info!("{:?}",item);
        }
    });
}
pub  fn rx_books_data_spawn(mut rx: Receiver<Books>){

    spawn(async move {
        // let map_books: = BTreeMap::new();
        loop {
            match rx.recv().await {
                Some(item) => {
                    info!("{:?}",item);
                }
                None => {
                    break;
                }
            }
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
        let books_vec = Vec::<[u64;400]>::new();
        let mut asks = [[0u64;400]];
        // println!("{}", asks.len());
        for x in asks[0] {
            println!("第{x}个")
        }
    }
    #[tokio::test]
    async fn ticker_test(){
        ticker("BTC-USDT-SWAP").await;
    }
}
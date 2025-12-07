use std::collections::{BTreeMap, HashMap};
use std::error;
use std::fs::File;
use std::sync::RwLock;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use sonic_rs::{from_str, JsonValueTrait};
use sonic_rs::writer::BufferedWriter;
use tokio::spawn;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver};
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio_tungstenite::tungstenite::Message::Text;
use tokio_tungstenite::tungstenite::{Error, Message, Utf8Bytes};
use okx::common::config::{get_ws_public};
use okx::common::rest_api::instruments;
use okx::common::utils::{log_init, send_str};
use okx::common::ws_api::{create_ws, login, order, subscribe, BookData, Books, OkxMessage, Ticker, TickerData, CHANNEL_BOOKS, CHANNEL_TICKERS};

#[tokio::main]
async fn main() ->Result<(), Box<dyn error::Error>>{
    log_init();
    let ws = create_ws(get_ws_public()).await?;
    let inst_id = "BTC-USDT-SWAP";
    let (mut tx, mut rx) = ws.split();
    tx.send(send_str(subscribe(CHANNEL_BOOKS,inst_id ).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_TICKERS,inst_id).as_str())).await?;
    let (book_channel_tx,book_channel_rx) = channel::<(Books,String,String)>(512);
    spawn(rx_books_spawn(book_channel_rx));
    let mut map_inst_id_price = HashMap::<String,String>::new();
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
                                                book_channel_tx.send((books,args.inst_id.clone(),match map_inst_id_price.get(&args.inst_id) { Some(o)=>{o.clone()},None=>"0".to_string()})).await.unwrap();
                                            }
                                            CHANNEL_TICKERS=>{
                                                let ticker = from_str::<Ticker>(&text).unwrap();
                                                for tick_data in ticker.data {
                                                    map_inst_id_price.insert(tick_data.inst_id,tick_data.last);
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
pub async fn rx_books_spawn(mut rx: Receiver<(Books,String,String)>){
        // let map_books: = BTreeMap::new();
        loop {
            match rx.recv().await {
                Some((b,id,p)) => {
                    // b.
                }
                None => {
                    break;
                }
            }
        }
}

#[cfg(test)]
mod test{
    use std::fs::File;
    use std::io::BufReader;
    use sonic_rs::{from_reader, from_str, to_writer_pretty};
    use sonic_rs::writer::BufferedWriter;
    use tokio::fs::read_to_string;
    use okx::common::rest_api::{instruments, ticker, OkxSwapInstrumentsResponse, SwapInstrument};
    use okx::common::utils::price_to_tick_int_str;

    #[tokio::test]
    async fn test_main(){
        let price_str = "91095.5";
        let file = File::open("data/instruments.json").unwrap();
        let reader = BufReader::new(file);
        let instruments: Vec<SwapInstrument> = from_reader::<BufReader<File>,Vec<SwapInstrument>>(reader).unwrap();
        for swap_instrument in instruments {
            if swap_instrument.settle_ccy.eq("USDT") {
                println!("{:?}",swap_instrument.inst_id);
            }
            // println!("{}",swap_instrument.settle_ccy)
            // break
        }

    }
    #[tokio::test]
    async fn ticker_test(){
        ticker("BTC-USDT-SWAP").await;
    }
}
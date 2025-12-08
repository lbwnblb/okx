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
use okx::common::utils::{get_min_sz, get_sz, log_init, price_to_tick_int_str, send_str};
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
}

pub async fn rx_books_spawn(mut rx: Receiver<(Books,String,String)>){
        // let map_books: = BTreeMap::new();
        let mut map_book_vec = HashMap::<(String,u64,u64),Vec<u64>>::new();

        loop {
            match rx.recv().await {
                Some((b,inst_id,p)) => {
                    // 消息的动作类型，例如 "snapshot" 或 "update"。
                    match b.action.as_str() {
                        "snapshot" => {
                            // let vec_asks = book_data.asks.into_iter().map(|vec_str| (price_to_tick_int_str(vec_str.get(0).unwrap(), get_sz(inst_id).unwrap()),price_to_tick_int_str(vec_str.get(1).unwrap(), get_min_sz(inst_id).unwrap()))).collect::<Vec<(u64,u64)>>();
                            // // let vec_bids = book_data.bids;
                            // let max_price = vec_asks.iter().map(|(price, _)| { price }).max().unwrap();
                            // let min_price = vec_asks.iter().map(|(price, _)| { price }).min().unwrap();
                            // let sub_price = max_price - min_price;
                            // println!("max_price: {}, min_price: {}, sub_price: {}", max_price, min_price, sub_price);
                            // let mut vec_price_v = vec![0u64;(sub_price+1) as usize];
                            // println!("{}", vec_price_v.len());
                            // println!("{}",max_price-min_price);
                            // vec_asks.iter().for_each(|(price,sz)| {
                            //     println!("index: {},price:{} min_price:{} sz: {}", price-min_price, price,min_price, sz);
                            //     vec_price_v[(price-min_price) as usize] = *sz;
                            // });
                            //
                            // map_book_vec.insert(("BTC-USDT-SWAP".to_string(),min_price.clone(),max_price.clone()),vec_price_v);
                            for b_d in b.data.into_iter() {
                                let vec_asks = b_d.asks.into_iter().map(|vec_str| (price_to_tick_int_str(vec_str.get(0).unwrap(), get_sz(&inst_id).unwrap()), price_to_tick_int_str(vec_str.get(1).unwrap(), get_min_sz(&inst_id).unwrap()))).collect::<Vec<(u64, u64)>>();
                                let max_price = vec_asks.iter().map(|(price, _)| { price }).max().unwrap();
                                let min_price = vec_asks.iter().map(|(price, _)| { price }).min().unwrap();
                                let interval = max_price - min_price;
                                let mut vec_price = vec![0u64;(interval+1) as usize];
                                vec_asks.iter().for_each(|(price,sz)| {
                                        vec_price[(price-min_price) as usize] = *sz;
                                });
                                map_book_vec.insert((inst_id.clone(),min_price.clone(),max_price.clone()),vec_price);
                                break
                            }
                        }
                        "update" =>{
                            for b_d in b.data.into_iter() {
                                for (inst_id_,min_price,max_price) in map_book_vec.keys() {

                                }
                            }
                        }
                        _ => {}
                    }
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
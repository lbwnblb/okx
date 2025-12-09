use std::collections::{BTreeMap, HashMap};
use std::error;
use std::fs::File;
use std::sync::RwLock;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use serde::de::Unexpected::Option;
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
use okx::common::ws_api::{create_ws, login, order, subscribe, BookData, Books, Books5, OkxMessage, Ticker, TickerData, CHANNEL_BOOKS, CHANNEL_BOOKS5, CHANNEL_TICKERS};

#[tokio::main]
async fn main() ->Result<(), Box<dyn error::Error>>{
    log_init();
    let ws = create_ws(get_ws_public()).await?;
    let inst_id = "BTC-USDT-SWAP";
    let (mut tx, mut rx) = ws.split();
    tx.send(send_str(subscribe(CHANNEL_BOOKS,inst_id ).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_TICKERS,inst_id).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_BOOKS5,inst_id).as_str())).await?;
    let (book_channel_tx,book_channel_rx) = channel::<(Books,String,String)>(512);
    spawn(rx_books_spawn(book_channel_rx));
    let mut map_inst_id_price = HashMap::<String,String>::new();
    loop {
        let result = rx.next().await;
        match result {
            None => {
                break;
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
                                                if book_channel_tx.send((books,args.inst_id.clone(),match map_inst_id_price.get(&args.inst_id) { Some(o)=>{o.clone()},None=>"0".to_string()})).await.is_err() {
                                                    error!("book channel closed");
                                                    break;
                                                }
                                            }
                                            CHANNEL_BOOKS5=>{
                                                let books5 = from_str::<Books5>(&text).unwrap();
                                                for book_data in books5.data {
                                                    let mut output = format!("========== BOOKS5: {} ==========\n", args.inst_id);
                                                    output.push_str("Asks (卖单):\n");
                                                    for (i, ask) in book_data.asks.iter().enumerate() {
                                                        if ask.len() >= 2 {
                                                            output.push_str(&format!("  [{}] Price: {}, Size: {}\n", i+1, ask[0], ask[1]));
                                                        }
                                                    }
                                                    output.push_str("Bids (买单):\n");
                                                    for (i, bid) in book_data.bids.iter().enumerate() {
                                                        if bid.len() >= 2 {
                                                            output.push_str(&format!("  [{}] Price: {}, Size: {}\n", i+1, bid[0], bid[1]));
                                                        }
                                                    }
                                                    output.push_str("======================================");
                                                    info!("{}", output);
                                                }
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

pub async fn rx_books_spawn(mut rx: Receiver<(Books,String,String)>){
        let mut map_book_vec_asks = HashMap::<(String,u64,u64),Vec<u64>>::new();
        let mut map_book_vec_bids = HashMap::<(String,u64,u64),Vec<u64>>::new();
        loop {
            match rx.recv().await {
                Some((b,inst_id,_p)) => {
                    // 消息的动作类型，例如 "snapshot" 或 "update"。
                    match b.action.as_str() {
                        "snapshot" => {
                            for b_d in b.data.into_iter() {
                                // 处理 asks
                                let vec_asks = b_d.asks.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str)).collect::<Vec<(u64, u64)>>();
                                let max_price = vec_asks.iter().map(|(price, _)| { price }).max().unwrap();
                                let min_price = vec_asks.iter().map(|(price, _)| { price }).min().unwrap();
                                let interval = max_price - min_price;
                                let mut vec_price = vec![0u64;(interval+1) as usize];
                                vec_asks.iter().for_each(|(price,sz)| {
                                        vec_price[(price-min_price) as usize] = *sz;
                                });
                                map_book_vec_asks.insert((inst_id.clone(),min_price.clone(),max_price.clone()),vec_price);
                                
                                // 处理 bids
                                let vec_bids = b_d.bids.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str)).collect::<Vec<(u64, u64)>>();
                                let max_price = vec_bids.iter().map(|(price, _)| { price }).max().unwrap();
                                let min_price = vec_bids.iter().map(|(price, _)| { price }).min().unwrap();
                                let interval = max_price - min_price;
                                let mut vec_price = vec![0u64;(interval+1) as usize];
                                vec_bids.iter().for_each(|(price,sz)| {
                                        vec_price[(price-min_price) as usize] = *sz;
                                });
                                map_book_vec_bids.insert((inst_id.clone(),min_price.clone(),max_price.clone()),vec_price);
                                break
                            }
                        }
                        "update" =>{
                            for b_d in b.data.into_iter() {
                                // 处理 asks 更新
                                let asks_p_v = b_d.asks.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str)).collect::<Vec<(u64, u64)>>();
                                let mut keys = map_book_vec_asks.keys().cloned().collect::<Vec<(String, u64, u64)>>();

                                for (p,v) in asks_p_v.iter() {
                                    let mut key = None;
                                    let mut flag_min = 0u64;
                                    let mut flag_max = 0u64;
                                    for (inst_id_for,min_price,max_price) in &keys{
                                        if !inst_id.eq(inst_id_for) {
                                            continue
                                        }
                                        if  min_price <= p && p <= max_price  {
                                            key = Some((inst_id_for.clone(),min_price.clone(),max_price.clone()));
                                        }
                                        if min_price > p {
                                            flag_min = min_price.clone();
                                        }
                                        if max_price < p {
                                            flag_max = max_price.clone();
                                        }
                                    }
                                    match key {
                                        None => {
                                            if flag_min != 0 && *p < flag_min {
                                                let interval = flag_min-p;
                                                if interval > 1000 {
                                                    continue
                                                }
                                                flag_max = flag_min-1;
                                                flag_min = flag_min-1000;
                                                let mut insert_vec = vec![0u64; 1000];
                                                insert_vec[(p-flag_min) as usize] = *v;
                                                map_book_vec_asks.insert((inst_id.clone(),flag_min,flag_max),insert_vec);
                                            }
                                            if flag_max != 0 && *p > flag_max {
                                                let interval = p-flag_max;
                                                if interval > 1000 {
                                                    continue
                                                }
                                                flag_min = flag_max+1;
                                                flag_max = flag_max+1000;
                                                let mut insert_vec = vec![0u64; 1000];
                                                insert_vec[(p-flag_min) as usize] = *v;
                                                map_book_vec_asks.insert((inst_id.clone(),flag_min,flag_max),insert_vec);
                                            }
                                            keys = map_book_vec_asks.keys().cloned().collect::<Vec<(String,u64,u64)>>();
                                        }
                                        Some((i,m_p,mi_p)) => {
                                            if let Some(vec) = map_book_vec_asks.get_mut(&(i,m_p,mi_p)) {
                                                vec[(p-m_p)as usize] = *v;
                                            }
                                        }
                                    }
                                }

                                // 处理 bids 更新
                                let bids_p_v = b_d.bids.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str)).collect::<Vec<(u64, u64)>>();
                                let mut keys = map_book_vec_bids.keys().cloned().collect::<Vec<(String, u64, u64)>>();

                                for (p,v) in bids_p_v.iter() {
                                    let mut key = None;
                                    let mut flag_min = 0u64;
                                    let mut flag_max = 0u64;
                                    for (inst_id_for,min_price,max_price) in &keys{
                                        if !inst_id.eq(inst_id_for) {
                                            continue
                                        }
                                        if  min_price <= p && p <= max_price  {
                                            key = Some((inst_id_for.clone(),min_price.clone(),max_price.clone()));
                                        }
                                        if min_price > p {
                                            flag_min = min_price.clone();
                                        }
                                        if max_price < p {
                                            flag_max = max_price.clone();
                                        }
                                    }
                                    match key {
                                        None => {
                                            if flag_min != 0 && *p < flag_min {
                                                let interval = flag_min-p;
                                                if interval > 1000 {
                                                    continue
                                                }
                                                flag_max = flag_min-1;
                                                flag_min = flag_min-1000;
                                                let mut insert_vec = vec![0u64; 1000];
                                                insert_vec[(p-flag_min) as usize] = *v;
                                                map_book_vec_bids.insert((inst_id.clone(),flag_min,flag_max),insert_vec);
                                            }
                                            if flag_max != 0 && *p > flag_max {
                                                let interval = p-flag_max;
                                                if interval > 1000 {
                                                    continue
                                                }
                                                flag_min = flag_max+1;
                                                flag_max = flag_max+1000;
                                                let mut insert_vec = vec![0u64; 1000];
                                                insert_vec[(p-flag_min) as usize] = *v;
                                                map_book_vec_bids.insert((inst_id.clone(),flag_min,flag_max),insert_vec);
                                            }
                                            keys = map_book_vec_bids.keys().cloned().collect::<Vec<(String,u64,u64)>>();
                                        }
                                        Some((i,m_p,mi_p)) => {
                                            if let Some(vec) = map_book_vec_bids.get_mut(&(i,m_p,mi_p)) {
                                                vec[(p-m_p)as usize] = *v;
                                            }
                                        }
                                    }
                                }
                                
                                // 打印前20档orderbook
                                print_orderbook(&inst_id, &map_book_vec_asks, &map_book_vec_bids);
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

fn as_bs_to_pv(inst_id: &String, vec_str: Vec<String>) -> (u64, u64) {
    let price_str = vec_str.get(0).unwrap();
    let sz_str = vec_str.get(1).unwrap();
    let price = price_to_tick_int_str(price_str, get_sz(&inst_id).unwrap());
    let sz = price_to_tick_int_str(sz_str, get_min_sz(&inst_id).unwrap());
    info!("as_bs_to_pv: price_str={}, sz_str={} -> price={}, sz={}", price_str, sz_str, price, sz);
    (price, sz)
}

fn tick_int_to_price(tick_int: u64, tick_size: &str) -> f64 {
    if !tick_size.contains(".") {
        return tick_int as f64;
    }
    let tick_size_split: Vec<&str> = tick_size.split('.').collect();
    let decimal_places = tick_size_split[1].len();
    tick_int as f64 / 10f64.powi(decimal_places as i32)
}

fn print_orderbook(
    inst_id: &str,
    map_book_vec_asks: &HashMap<(String, u64, u64), Vec<u64>>,
    map_book_vec_bids: &HashMap<(String, u64, u64), Vec<u64>>,
) {
    let tick_size = get_sz(inst_id).unwrap();
    let min_sz = get_min_sz(inst_id).unwrap();
    
    // 收集所有 asks
    let mut asks_list: Vec<(u64, u64)> = Vec::new();
    for ((i, min_price, _max_price), vec) in map_book_vec_asks {
        if i == inst_id {
            for (idx, &sz) in vec.iter().enumerate() {
                if sz > 0 {
                    let price = min_price + idx as u64;
                    asks_list.push((price, sz));
                }
            }
        }
    }
    asks_list.sort_by_key(|(price, _)| *price);
    
    // 收集所有 bids
    let mut bids_list: Vec<(u64, u64)> = Vec::new();
    for ((i, min_price, _max_price), vec) in map_book_vec_bids {
        if i == inst_id {
            for (idx, &sz) in vec.iter().enumerate() {
                if sz > 0 {
                    let price = min_price + idx as u64;
                    bids_list.push((price, sz));
                }
            }
        }
    }
    bids_list.sort_by_key(|(price, _)| std::cmp::Reverse(*price));
    let mut output = String::new();
    for (price, sz) in bids_list.iter().take(5) {
        output.push_str(&format!("bids price {price} sz {sz}\n"));
    }
    for (price, sz) in asks_list.iter().take(5) {
        output.push_str(&format!("asks price {price} sz {sz}\n"));
    }
    if !output.is_empty() {
        info!("{}", output.trim_end());
    }
    
    info!("====================================\n");
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
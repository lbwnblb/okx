use std::collections::{BTreeMap, HashMap};
use std::error;
use std::fs::File;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use log::{error, info};
use serde::de::Unexpected::Option;
use sonic_rs::{from_str, JsonValueTrait};
use sonic_rs::writer::BufferedWriter;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver};
use tokio::sync::mpsc::error::{SendError, TrySendError};
use tokio_tungstenite::tungstenite::Message::Text;
use tokio_tungstenite::tungstenite::{Bytes, Error, Message, Utf8Bytes};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use okx::common::config::{get_ws_private, get_ws_public};
use okx::common::rest_api::instruments;
use okx::common::utils::{get_inst_id_code, get_min_sz, get_sz, log_init, order_id_str, price_to_tick_int_str, send_str, tick_int_to_price_str};
use okx::common::ws_api::{create_ws, login, order, order_market, subscribe, BookData, Books, Books5, OkxMessage, OrderType, Side, Ticker, TickerData, CHANNEL_BBO_TBT, CHANNEL_BOOKS, CHANNEL_BOOKS5, CHANNEL_TICKERS};

static ORDER_COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct TaskFn;
impl TaskFn {
    pub async fn rx_books(mut rx: Receiver<(Utf8Bytes,String,u8)>){
        let mut map_book_vec_asks = HashMap::<(String,u64,u64),Vec<u64>>::new();
        let mut map_book_vec_bids = HashMap::<(String,u64,u64),Vec<u64>>::new();
        loop {
            match rx.recv().await {
                Some((b,inst_id,task_id)) => {
                    let min_sz = get_min_sz(&inst_id).unwrap();
                    let sz = get_sz(&inst_id).unwrap();
                    match task_id {
                        0 => {
                            let b = from_str::<Books>(&b).unwrap();
                            // 消息的动作类型，例如 "snapshot" 或 "update"。
                            match b.action.as_str() {
                                "snapshot" => {
                                    for b_d in b.data.into_iter() {
                                        // 处理 asks
                                        let vec_asks = b_d.asks.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str,sz)).collect::<Vec<(u64, u64)>>();
                                        let max_price = vec_asks.iter().map(|(price, _)| { price }).max().unwrap();
                                        let min_price = vec_asks.iter().map(|(price, _)| { price }).min().unwrap();
                                        let interval = max_price - min_price;
                                        let mut vec_price = vec![0u64;(interval+1) as usize];
                                        vec_asks.iter().for_each(|(price,sz)| {
                                            vec_price[(price-min_price) as usize] = *sz;
                                        });
                                        map_book_vec_asks.insert((inst_id.clone(),min_price.clone(),max_price.clone()),vec_price);

                                        // 处理 bids
                                        let vec_bids = b_d.bids.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str,sz)).collect::<Vec<(u64, u64)>>();
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
                                        let asks_p_v = b_d.asks.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str,sz)).collect::<Vec<(u64, u64)>>();
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
                                        let bids_p_v = b_d.bids.into_iter().map(|vec_str| as_bs_to_pv(&inst_id, vec_str,sz)).collect::<Vec<(u64, u64)>>();
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
                                    }
                                }
                                _ => {}
                            }
                        },
                        1 => {

                        },
                        _ => {}
                    }

                }
                None => {
                    break;
                }
            }
        }
    }
    pub async fn rx_order(mut rx:Receiver<String>, mut tx_ws: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>){
        while let Some(b) = rx.recv().await {
            if let Err( e) = tx_ws.send(send_str(&b)).await{
                error!("发送失败 {} {}",b,e);
            }
        }
    }
    pub async fn rx_ws_order(mut rx_order_ws: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>){
        while let Some(b) = rx_order_ws.next().await {
            match b {
                Ok(Text(s)) => {
                    info!("{}",s.as_str());
                },
                _ => {}
            }
        }
    }
}

#[tokio::main]
async fn main() ->Result<(), Box<dyn error::Error>>{
    log_init();
    let ws = create_ws(get_ws_public()).await?;

    let ws_order = create_ws(get_ws_private()).await?;
    let (mut tx_order_ws, rx_order_ws) = ws_order.split();
    tx_order_ws.send(send_str(&login())).await.unwrap();
    let inst_id = "ETH-USDT-SWAP";
    let (mut tx, mut rx) = ws.split();
    tx.send(send_str(subscribe(CHANNEL_BOOKS,inst_id ).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_TICKERS,inst_id).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_BOOKS5,inst_id).as_str())).await?;
    tx.send(send_str(subscribe(CHANNEL_BBO_TBT,inst_id).as_str())).await?;
    let (book_channel_tx,book_channel_rx) = channel::<(Utf8Bytes,String,u8)>(512);
    let (tx_order_channel,rx_order_channel) = channel::<String>(512);
    spawn(TaskFn::rx_books(book_channel_rx));
    spawn(TaskFn::rx_order(rx_order_channel,tx_order_ws));
    spawn(TaskFn::rx_ws_order(rx_order_ws));

    let mut is_send_order = false;
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
                                                // let books = from_str::<Books>(&text).unwrap();
                                                if book_channel_tx.send((text,args.inst_id.clone(),0)).await.is_err() {
                                                    error!("book channel closed");
                                                    break;
                                                }
                                            }
                                            CHANNEL_BOOKS5=>{
                                                if book_channel_tx.send((text,args.inst_id.clone(),1)).await.is_err(){
                                                    error!("book channel closed");
                                                    break;
                                                };
                                            }
                                            CHANNEL_TICKERS=>{

                                            }
                                            CHANNEL_BBO_TBT=>{
                                                info!("CHANNEL_BBO_TBT {}",text);
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


fn as_bs_to_pv(inst_id: &String, vec_str: Vec<String>,sz:&str) -> (u64, u64) {
    let price_str = vec_str.get(0).unwrap();
    let sz_str = vec_str.get(1).unwrap();
    let price = price_to_tick_int_str(price_str, sz);
    let sz = price_to_tick_int_str(sz_str, sz);
    // info!("as_bs_to_pv: price_str={}, sz_str={} -> price={}, sz={}", price_str, sz_str, price, sz);
    (price, sz)
}




#[cfg(test)]
mod test{
    use okx::common::utils::get_quantity_sz;
    use super::*;

    #[tokio::test]
    async fn quantity_test(){
        let inst_id = "BTC-USDT-SWAP";
        println!("{}", get_quantity_sz(inst_id,"1.0"));
    }



    #[tokio::test]
    async fn order_test() -> Result<(), Box<dyn std::error::Error>>{
        log_init();
        let ws_order = create_ws(get_ws_private()).await?;
        let (mut tx, mut rx) = ws_order.split();
        let inst_id = "BTC-USDT-SWAP";
        let order_id = ORDER_COUNTER.fetch_add(1, Ordering::Relaxed).to_string();

        let market_order = order_market(&order_id, Side::BUY,inst_id,&get_quantity_sz(inst_id,"1.0"));
        tx.send(send_str(login().as_str())).await?;
        let mut is_send_order = false;
        loop {
            let result = rx.next().await.unwrap();
            match result {
                Ok(Text(text)) => {
                    info!("{}",text.as_str());
                    if !is_send_order {
                        tx.send(send_str(&market_order)).await?;
                        is_send_order = true;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }


}
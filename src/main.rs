use sonic_rs::{JsonContainerTrait, JsonValueTrait};
use okx::common::rest_api::instruments;
use okx::common::utils::log_init;
use okx::common::ws_api::create_ws;

#[tokio::main]
async fn main() ->Result<(), Box<dyn std::error::Error>>{
    log_init();
    let value = instruments().await?;
    let data = value.get("data").unwrap();
    let data = data.as_array().unwrap();
    data.iter().for_each(|item|{
        println!("{}", item);
    });
    Ok(())
}

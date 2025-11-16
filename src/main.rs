use okx::common::rest_api::instruments;
use okx::common::utils::log_init;
use okx::common::ws_api::create_ws;

#[tokio::main]
async fn main() ->Result<(), Box<dyn std::error::Error>>{
    log_init();
    let value = instruments().await?;
    value


    Ok(())
}

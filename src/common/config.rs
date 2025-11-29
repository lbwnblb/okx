use std::sync::LazyLock;

pub const WS_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
pub const WS_SIMULATION_URL: &str = "wss://wspap.okx.com:8443/ws/v5/public";
pub const REST_URL: &str = "https://www.okx.com";
pub const REST_SIMULATION_URL: &str = "https://www.okx.com";
pub static  OKX_API_KEY:LazyLock<String> = LazyLock::new(|| std::env::var("OKX_API_KEY").expect("OKX_API_KEY not set"));
pub static  OKX_SECRET_KEY:LazyLock<String> = LazyLock::new(|| std::env::var("OKX_SECRET_KEY").expect("OKX_SECRET_KEY not set"));
pub static  OK_ACCESS_PASSPHRASE:LazyLock<String> = LazyLock::new(|| std::env::var("OK_ACCESS_PASSPHRASE").expect("OK_ACCESS_PASSPHRASE not set"));
/// 模拟API KEY
pub static  OKX_SIMULATION_API_KEY:LazyLock<String> = LazyLock::new(|| std::env::var("OKX_SIMULATION_API_KEY").expect("OKX_SIMULATION_API_KEY not set"));
/// 模拟API Passphrase
pub static  OK_SIMULATION_ACCESS_PASSPHRASE:LazyLock<String> = LazyLock::new(|| std::env::var("OK_SIMULATION_ACCESS_PASSPHRASE").expect("OK_SIMULATION_ACCESS_PASSPHRASE not set"));
/// 模拟API Secret Key
pub static  OKX_SIMULATION_SECRET_KEY:LazyLock<String> = LazyLock::new(|| std::env::var("OKX_SIMULATION_SECRET_KEY").expect("OKX_SIMULATION_SECRET_KEY not set"));
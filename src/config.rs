use std::sync::Mutex;

use config::ConfigError;
use lazy_static::lazy_static;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub wxcorp_id: String,
    pub wxcorp_secret: String,
    pub wxcorp_app_id: String,
    pub wxcorp_token: String,
    pub wxcorp_aes_key: String,
    pub db_path: String,
    pub listen: String,
}

impl Config {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();
        match settings.merge(config::File::with_name(path)) {
            Ok(_) => settings.try_into(),
            Err(err) => Err(err),
        }
    }
}


// 全局配置对象
lazy_static! {
    pub static ref CONFIG_FILE: Mutex<String> = Mutex::new("config.toml".to_string());
    pub static ref CONFIG: Config = match Config::new(&CONFIG_FILE.lock().unwrap()) {
        Ok(cfg) => cfg,
        Err(err) => panic!("{:?}", err),
    };
}

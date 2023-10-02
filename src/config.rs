use std::sync::Mutex;

use config::{ConfigError};
use lazy_static::lazy_static;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub wxcorp_id: String,
    pub wxcorp_secret: String,
    pub wxcorp_app_id: String,
    pub wxcorp_token: String,
    pub wxcorp_encoding_aes_key: String,
    pub db_path: String,
    pub listen: String,
    pub database_host: String,
    pub database_port: u16,
    pub database_user: String,
    pub database_password: String,
    pub database_name: String,
    pub database_type: String,
}

impl AppConfig {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let builder = config::Config::builder().add_source(config::File::with_name(path));
        match builder.build(){
            Ok(config) => config.try_deserialize(),
            Err(err) => Err(err),
        }
    }
}


// 全局配置对象
lazy_static! {
    pub static ref CONFIG_FILE: Mutex<String> = Mutex::new("config.toml".to_string());
    pub static ref CONFIG: AppConfig = match AppConfig::new(&CONFIG_FILE.lock().unwrap()) {
        Ok(cfg) => cfg,
        Err(err) => panic!("{:?}", err),
    };
}

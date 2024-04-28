use std::error::Error;

use config::Config;
use context::SERVICES;
use database::DbService;
use redis_io::Redis;

use crate::user::UserService;
use crate::wx_corp::WxCorpService;

pub async fn init_services(config: &Config) -> Result<(), Box<dyn Error>> {
    SERVICES.set(DbService::create(config).await);

    Redis::init_from_config(config);
    
    SERVICES.set(UserService::default());

    SERVICES.set(WxCorpService::default());

    Ok(())
}

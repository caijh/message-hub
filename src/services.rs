use std::error::Error;

use config::Config;
use context::SERVICES;
use database::DbService;
use redis_util::{Redis, RedisConfig};

use crate::user::UserService;
use crate::wx_corp::WxCorpService;

pub async fn init_services(config: &Config) -> Result<(), Box<dyn Error>> {
    let database_service = DbService::create(config).await;
    SERVICES.set(database_service);

    let user_service = UserService::default();
    SERVICES.set(user_service);

    let wx_corp = WxCorpService::default();
    SERVICES.set(wx_corp);

    let redis_config = RedisConfig::get_redis_config(config);
    Redis::init(&redis_config);

    Ok(())
}

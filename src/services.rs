use std::error::Error;
use config::Config;

use state::TypeMap;

use crate::database::DatabaseService;
use crate::user::UserService;
use crate::wx_corp::WxCorpService;

lazy_static::lazy_static! {
    pub static ref SERVICES: TypeMap![Send + Sync] = <TypeMap![Send + Sync]>::new();
}

pub async fn init_services(config: &Config) -> Result<(), Box<dyn Error>> {
    let database_service = DatabaseService::new(config).await;
    SERVICES.set(database_service);
    let user_service = UserService::new(config);
    SERVICES.set(user_service);
    let wx_corp = WxCorpService::new(config);
    SERVICES.set(wx_corp);
    Ok(())
}

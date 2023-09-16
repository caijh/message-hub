use std::error::Error;

use state::TypeMap;

use crate::database::DatabaseService;
use crate::user::UserService;
use crate::wx_corp::WxCorpService;

lazy_static::lazy_static! {
    pub static ref SERVICES: TypeMap![Send + Sync] = <TypeMap![Send + Sync]>::new();
}

pub async fn init_services() -> Result<(), Box<dyn Error>> {
    let database_service = DatabaseService::new().await;
    SERVICES.set(database_service);
    let user_service = UserService::new();
    SERVICES.set(user_service);
    let wx_corp = WxCorpService::default();
    SERVICES.set(wx_corp);
    Ok(())
}

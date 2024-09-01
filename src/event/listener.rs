use application::application::{Application, RustApplication};
use application::context::application_context::ApplicationContext;
use application::context::application_event::{ApplicationEvenType, ApplicationEvent};
use application::context::application_listener::ApplicationListener;
use application::env::property_resolver::PropertyResolver;
use database_common::connection::DbConnection;
use database_mysql_seaorm::Dao;
use redis_io::{Redis, RedisConfig};

use crate::service::user::UserService;
use crate::service::wx_corp::WxCorpService;
use async_trait::async_trait;

pub struct ApplicationContextInitializedListener;

#[async_trait]
impl ApplicationListener for ApplicationContextInitializedListener {
    fn is_support(&self, event: &dyn ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::ContextInitialized
    }

    async fn on_application_event(
        &self,
        application: &RustApplication,
        _event: &dyn ApplicationEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let application_context = application.get_application_context();

        let environment = application_context.get_environment().await;
        let db_connection = environment
            .get_property::<DbConnection>("database")
            .unwrap();
        let dao = Dao::new(db_connection).await;
        application_context.context.set(dao);

        let redis_config = environment.get_property::<RedisConfig>("redis");
        if redis_config.is_some() {
            Redis::init(&redis_config.unwrap())
        }

        application_context.context.set(UserService::default());

        application_context.context.set(WxCorpService::default());
        Ok(())
    }
}

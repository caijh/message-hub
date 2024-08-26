use application::application::Application;
use application::application_context::ApplicationContext;
use application::environment::Environment;
use application::{
    application_event::ApplicationEvenType, application_listener::ApplicationListener,
};
use database_common::connection::DbConnection;
use database_mysql_seaorm::Dao;

use crate::{user::UserService, wx_corp::WxCorpService};
use async_trait::async_trait;

pub struct ApplicationContextInitializedListener;

#[async_trait]
impl ApplicationListener for ApplicationContextInitializedListener {
    fn is_support(&self, event: &dyn application::application_event::ApplicationEvent) -> bool {
        event.get_event_type() == ApplicationEvenType::ContextInitialized
    }

    async fn on_application_event(
        &self,
        application: &application::application::RustApplication,
        _event: &dyn application::application_event::ApplicationEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let application_context = application.get_application_context();

        let environment = application_context.get_environment();
        let db_connection = environment
            .get_property::<DbConnection>("database")
            .unwrap();
        let dao = Dao::new(db_connection).await;
        application_context.context.set(dao);

        application_context.context.set(UserService::default());

        application_context.context.set(WxCorpService::default());
        Ok(())
    }
}

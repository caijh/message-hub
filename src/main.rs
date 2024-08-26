use application::application::{Application, RustApplication};
use message_hub::{controller::router::RouterContextInitializer, event::listener::ApplicationContextInitializedListener};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .add_listener(Box::new(ApplicationContextInitializedListener))
        .await;

    application.add_servlet_context_initializer(Box::new(RouterContextInitializer)).await;

    application.run().await?;

    Ok(())
}

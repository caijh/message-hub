use application_boot::application::{Application, RustApplication};
use message_hub::event::listener::ApplicationContextInitializedListener;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let application = RustApplication::default();

    application
        .add_listener(Box::new(ApplicationContextInitializedListener))
        .await;

    application.run().await?;

    Ok(())
}

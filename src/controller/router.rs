use application::initializer::ServletContextInitializer;
use axum::{
    routing::{get, post},
    Router,
};
use web::health::health_routers;

use crate::controller::index::{do_get_wx_corp_receive, do_post_wx_corp_receive, handle_message_detail, handle_send_message};

pub struct RouterContextInitializer;

impl ServletContextInitializer for RouterContextInitializer {
    fn initialize(&self, router: Router) -> Router {
        router
            .nest("/", routers())
            .nest("/health", health_routers())
    }
}

fn routers() -> Router {
    Router::new()
        .route("/", get(do_get_wx_corp_receive))
        .route("/", post(do_post_wx_corp_receive))
        .route("/send/:username", post(handle_send_message))
        .route("/message/:id", get(handle_message_detail))
}

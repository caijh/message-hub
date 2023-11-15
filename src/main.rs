use std::io::prelude::*;

use actix_web::{App, HttpServer, web};
use actix_web::web::get;
use clap::{arg, Command, crate_version};
use configuration::Configuration;
use handlebars::Handlebars;
use logger::{Logger, LoggerConfig};
use tracing::{error, info};

use message_hub::handler;
use message_hub::handler::{stop, StopHandle};
use message_hub::services::init_services;

mod storage;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 参数处理
    let matches = Command::new("Server Tan")
        .version(crate_version!())
        .author("caijunhui. <caijh@gmail.com>")
        .about("Message Hub...")
        .args(&[
            arg!(-c --config <FILE> "Sets a custom config file")
        ])
        .get_matches();

    let path = if let Some(c) = matches.get_one::<String>("config") {
        c
    } else {
        "./config.toml"
    };

    Configuration::load(path).await.expect("Load config failed");

    let config = Configuration::get_config().await.clone();

    let mut logger = Logger::new(&LoggerConfig {
        enabled: config.get_bool("logger.enabled").unwrap_or_default(),
        level: config.get_string("logger.level").unwrap_or("info".to_string()),
        file: config.get_string("application.name").unwrap_or("info".to_string()) + ".log",
        log_dir: "./logs".to_string(),
    });
    logger.init();

    // 初始化Service
    init_services(&config).await.expect("init services failed");

    let stop_handle = web::Data::new(StopHandle::default());

    let addr = config.get_string("listen").unwrap();
    info!("Listening on https://{}", addr);
    let mut hbars = Handlebars::new();
    hbars
        .register_templates_directory(".html", "./static/")
        .unwrap();
    let hbars_ref = web::Data::new(hbars);
    let server = HttpServer::new({
        let stop_handle = stop_handle.clone();
        move || {
            App::new()
                .app_data(hbars_ref.clone()).app_data(stop_handle.clone())
                .service(stop)
                .route("/", web::get().to(handler::do_get_wx_corp_receive))
                .route("/", web::post().to(handler::do_post_wx_corp_receive))
                .route("/send/{username}", web::post().to(handler::handle_send_message))
                .route("/message/{id}", web::get().to(handler::handler_message_detail))
                .route("/health/check", get().to(handler::health_check))
        }
    })
        .bind(addr)?
        .run();

    // register the server handle with the stop handle
    stop_handle.register(server.handle());

    // Register with Consul
    if let Err(err) = registration::register(&config).await {
        error!("Failed to register with Consul: {}", err);
        // Shut down Actix Web server if Consul registration fails
        stop_handle.stop(true).await;
    }

    server.await?;

    stop_handle.stop(true).await;

    Ok(())
}

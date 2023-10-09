use std::io::prelude::*;

use actix_web::{App, HttpServer, web};
use actix_web::web::get;
use clap::{arg, Command, crate_version};
use configuration::Configuration;
use handlebars::Handlebars;
use log::{debug, info};

use message_hub::handler;
use message_hub::handler::StopHandle;
use message_hub::services::init_services;


mod storage;


// 初始化日志，自定义了日志格式
fn init_log() {
    use chrono::Local;

    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}:{}:{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                record.file().unwrap_or(""),
                record.line().unwrap_or(0),
                &record.args()
            )
        })
        .init();

    info!("env_logger initialized.");
}

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
        debug!("Value for config: {}", c);
        c
    } else {
        "./config.toml"
    };

    Configuration::load(path).await.expect("load config failed");
    #[allow(clippy::await_holding_lock)]
    let config = Configuration::get_config().await;

    // 初始化日志
    init_log();

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
    let server = HttpServer::new(move || {
        App::new()
            .app_data(hbars_ref.clone())
            .route("/", web::get().to(handler::do_get_wx_corp_receive))
            .route("/", web::post().to(handler::do_post_wx_corp_receive))
            .route("/send/{username}", web::post().to(handler::handle_send_message))
            .route("/message/{id}", web::get().to(handler::handler_message_detail))
            .route("/health/check", get().to(handler::health_check))
    })
        .bind(addr)?
        .run();

    // register the server handle with the stop handle
    stop_handle.register(server.handle());

    // Register with Consul
    if let Err(err) = registration::register(&config).await {
        eprintln!("Failed to register with Consul: {}", err);
        // Shut down Actix Web server if Consul registration fails
        stop_handle.stop(true).await;
    }
    server.await
}

use std::io::prelude::*;

use actix_web::{App, HttpServer, web};
use clap::crate_version;
use handlebars::Handlebars;
use log::{debug, info};

use messagehub::handler;
use messagehub::services::init_services;

use crate::config::CONFIG_FILE;

mod config;
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
    // 初始化日志
    init_log();

    // 初始化Service
    init_services().await.expect("init services failed");

    // 参数处理
    let matches = clap::App::new("Server Tan")
        .version(crate_version!())
        .author("caijunhui. <caijh@gmail.com>")
        .about("Message Hub...")
        .args_from_usage("-c, --config=[FILE] 'Sets a custom config file'")
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        debug!("Value for config: {}", c);
        *CONFIG_FILE.lock().unwrap() = c.to_string();
    }

    info!("Listening on https://{}", config::CONFIG.listen);
    let mut hbars = Handlebars::new();
    hbars
        .register_templates_directory(".html", "./static/")
        .unwrap();
    let hbars_ref = web::Data::new(hbars);
    HttpServer::new(move || {
        App::new()
            .app_data(hbars_ref.clone())
            .route("/", web::get().to(handler::do_get_wx_corp_receive))
            .route("/", web::post().to(handler::do_post_wx_corp_receive))
            .route("/send/{username}", web::post().to(handler::handle_send_message))
            .route("/message/{id}", web::get().to(handler::handler_message_detail))
    })
        .bind(&config::CONFIG.listen)?
        .run()
        .await
}

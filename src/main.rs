#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use std::io::prelude::*;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use actix_web::web::{Json, Path};

use messagehub::{auth, wx_corp};
use messagehub::auth::Signature;

use crate::config::CONFIG_FILE;

mod config;
mod storage;
mod user;

mod message;

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

#[derive(Deserialize, Debug)]
struct AuthEchoInfo {
    signature: String,
    timestamp: String,
    nonce: String,
    echostr: String,
}


#[derive(Deserialize, Debug)]
struct User {
    username: String,
}


fn do_wx_corp(msg: message::TextCardMessage) -> String {
    let json = serde_json::to_string(&msg).unwrap();
    let result = wx_corp::INTERFACE.send(&json);
    serde_json::to_string(&result).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgReqBody {
    pub content: String,
}

fn wx_post(user: Path<User>, query: web::Query<Signature>, message: Json<MsgReqBody>) -> impl Responder {
    debug!("POST /send/{}", user.username);
    let signature = &query.signature;
    let timestamp = &query.timestamp;
    let nonce = &query.nonce;
    if !auth::check_signature(signature, timestamp, nonce) {
        debug!("auth failed!");
        return HttpResponse::Forbidden().finish();
    }
    debug!("auth pass!");
    debug!("msg:{}", message.content);

    let username = &user.username;
    let user = user::INTERFACE.get_user(username);
    match user {
        Ok(_) => {
            let msg = message::parse_message(username.as_str(), &message.content);
            HttpResponse::Ok().body(do_wx_corp(msg))
        }
        Err(_) => {
            HttpResponse::Forbidden().finish()
        }
    }
}

fn wx_auth(query: web::Query<AuthEchoInfo>) -> impl Responder {
    debug!("get /auth");
    debug!("query:{:?}", query);
    let signature = &query.signature;
    let timestamp = &query.timestamp;
    let nonce = &query.nonce;
    let echostr = &query.echostr;
    debug!("echostr:{}", echostr);
    if auth::check_signature(signature, timestamp, nonce) {
        debug!("auth pass!");
        HttpResponse::Ok().body(echostr)
    } else {
        debug!("auth failed!");
        HttpResponse::Forbidden().finish()
    }
}

fn main() {
    // 初始化日志
    init_log();

    // 参数处理
    let matches = clap::App::new("Server Tan")
        .version(crate_version!())
        .author("Caijh. <caijh@gmail.com>")
        .about("Message Hub...")
        .args_from_usage("-c, --config=[FILE] 'Sets a custom config file'")
        .get_matches();

    if let Some(c) = matches.value_of("config") {
        debug!("Value for config: {}", c);
        *CONFIG_FILE.lock().unwrap() = c.to_string();
    }

    info!("Listening on https://{}", config::CONFIG.listen);

    HttpServer::new(|| {
        App::new()
            .route("/auth", web::get().to(wx_auth))
            .route("/send/{username}", web::post().to(wx_post))
    })
        .bind(&config::CONFIG.listen)
        .unwrap()
        .run()
        .unwrap();
}

use actix_web::{get, HttpResponse, Responder, web};
use actix_web::dev::ServerHandle;
use actix_web::web::{Json, Path};
use configuration::Configuration;
use handlebars::Handlebars;
use parking_lot::Mutex;
use serde_derive::{Deserialize, Serialize};
use tracing::debug;

use crate::{auth, message, wx_corp};
use crate::auth::Signature;
use crate::message::send_by_wx_corp;
use crate::services::SERVICES;
use crate::user::UserService;

#[derive(Serialize, Deserialize, Debug)]
pub struct WxCorpJoinValidate {
    msg_signature: String,
    timestamp: String,
    nonce: String,
    #[serde(rename = "echostr")]
    echo_str: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgReqBody {
    pub title: Option<String>,
    pub content: String,
}

pub async fn do_get_wx_corp_receive(query: web::Query<WxCorpJoinValidate>) -> impl Responder {
    debug!("get /");
    debug!("query:{:?}", query);

    let msg_signature = &query.msg_signature;
    let time_stamp = &query.timestamp;
    let nonce = &query.nonce;
    let echo_str = &query.echo_str;
    let config = Configuration::get_config().await;
    let token = config.get_string("wxcorp_token").unwrap();
    let aes_key = config.get_string("wxcorp_encoding_aes_key").unwrap();
    let result = wx_corp::verify_url(msg_signature, token.as_str(),time_stamp, nonce, echo_str, aes_key.as_str());
    match result {
        Ok(r) => HttpResponse::Ok().body(r),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}


pub async fn do_post_wx_corp_receive() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn handle_send_message(user: Path<User>, query: web::Query<Signature>, message: Json<MsgReqBody>) -> impl Responder {
    debug!("POST /send/{}", user.username);
    let signature = &query.signature;
    let timestamp = &query.timestamp;
    let nonce = &query.nonce;
    let title = message.title.clone().unwrap_or_default();
    let content = message.content.as_str();
    let config = Configuration::get_config().await.clone();
    let token = config.get_string("wxcorp_token").unwrap();
    if !auth::check_signature(signature, token.as_str(), timestamp, nonce, content) {
        debug!("auth failed!");
        return HttpResponse::Forbidden().finish();
    }
    debug!("auth pass!");
    debug!("msg:{}", message.content);

    let username = &user.username;
    let user = SERVICES.get::<UserService>().get_user(username).await;
    match user {
        Ok(_) => {
            let app_id = config.get_string("wxcorp_app_id").unwrap();
            let response = send_by_wx_corp(app_id.as_str(),username, title.as_str(), content).await;
            HttpResponse::Ok().body(response)
        }
        Err(e) => {
            debug!("Get user info error, {:?}", e);
            HttpResponse::Forbidden().finish()
        }
    }
}

pub async fn handler_message_detail(hb: web::Data<Handlebars<'_>>, id: Path<String>) -> impl Responder {
    let message = message::get_message_detail(&id).await;
    let body = hb.render("message", &message).unwrap();
    HttpResponse::Ok().body(body)
}


#[get("/stop/{graceful}")]
async fn stop(graceful: Path<bool>, stop_handle: web::Data<StopHandle>) -> HttpResponse {
    stop_handle.stop(graceful.to_owned()).await;
    HttpResponse::NoContent().finish()
}

#[derive(Default)]
pub struct StopHandle {
    inner: Mutex<Option<ServerHandle>>,
}

impl StopHandle {
    /// Sets the server handle to stop.
    pub fn register(&self, handle: ServerHandle) {
        *self.inner.lock() = Some(handle);
    }

    /// Sends stop signal through contained server handle.
    pub async fn stop(&self, graceful: bool) {
        let _ = registration::deregister().await;
        #[allow(clippy::let_underscore_future)]
        let _ = self.inner.lock().as_ref().unwrap().stop(graceful);
    }
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}

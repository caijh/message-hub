use actix_web::{HttpResponse, post, Responder, web};
use actix_web::dev::ServerHandle;
use actix_web::web::{Json, Path};
use handlebars::Handlebars;
use log::debug;
use parking_lot::Mutex;
use serde_derive::{Deserialize, Serialize};

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
    echostr: String,
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
    let echo_str = &query.echostr;
    let result = wx_corp::verify_url(msg_signature, time_stamp, nonce, echo_str);
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
    let content = message.content.as_str();
    if !auth::check_signature(signature, timestamp, nonce, content) {
        debug!("auth failed!");
        return HttpResponse::Forbidden().finish();
    }
    debug!("auth pass!");
    debug!("msg:{}", message.content);

    let username = &user.username;
    let user = SERVICES.get::<UserService>().get_user(username).await;
    match user {
        Ok(_) => {
            let response = send_by_wx_corp(username, content).await;
            HttpResponse::Ok().body(response)
        }
        Err(_) => {
            HttpResponse::Forbidden().finish()
        }
    }
}

pub async fn handler_message_detail(hb: web::Data<Handlebars<'_>>, id: Path<String>) -> impl Responder {
    let message = message::get_message_detail(&id).await;
    let body = hb.render("message", &message).unwrap();
    HttpResponse::Ok().body(body)
}


#[post("/stop/{graceful}")]
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

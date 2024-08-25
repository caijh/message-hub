use application::application::APPLICATION_CONTEXT;
use application::environment::{ApplicationEnvironment, Environment};
use askama::Template;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use tracing::debug;
use web::response::RespBody;

use crate::auth::Signature;
use crate::message_svc::send_by_wx_corp;
use crate::user::UserService;
use crate::{auth, message_svc, wx_corp};

#[derive(Serialize, Deserialize, Debug)]
pub struct WxCorpJoinValidate {
    msg_signature: String,
    timestamp: String,
    nonce: String,
    #[serde(rename = "echostr")]
    echo_str: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgReqBody {
    pub title: Option<String>,
    pub content: String,
}

pub async fn do_get_wx_corp_receive(Query(params): Query<WxCorpJoinValidate>) -> impl IntoResponse {
    debug!("get /");
    debug!("query:{:?}", params);

    let msg_signature = &params.msg_signature;
    let time_stamp = &params.timestamp;
    let nonce = &params.nonce;
    let echo_str = &params.echo_str;
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.environment.read().await;
    let token = environment.get_property::<String>("wxcorp_token").unwrap();
    let aes_key = environment
        .get_property::<String>("wxcorp_encoding_aes_key")
        .unwrap();
    let result = wx_corp::verify_url(
        msg_signature,
        token.as_str(),
        time_stamp,
        nonce,
        echo_str,
        aes_key.as_str(),
    );
    RespBody::from_result(&result).response()
}

pub async fn do_post_wx_corp_receive() -> impl IntoResponse {
    RespBody::from(&"".to_string()).response()
}

pub async fn handle_send_message(
    Path(username): Path<String>,
    Query(query): Query<Signature>,
    Json(message): Json<MsgReqBody>,
) -> impl IntoResponse {
    debug!("POST /send/{}", username);
    let signature = &query.signature;
    let timestamp = &query.timestamp;
    let nonce = &query.nonce;
    let content = message.content.as_str();
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.environment.read().await;
    let token = environment.get_property::<String>("wxcorp_token").unwrap();
    if !auth::check_signature(signature, token.as_str(), timestamp, nonce, content) {
        debug!("auth failed!");
        return (StatusCode::FORBIDDEN, "auth failed").into_response();
    }
    debug!("auth pass!");
    debug!("msg:{}", message.content);

    let title = message.title.clone().unwrap_or_default();
    let user = send_to_user(&username, &title, content, &environment).await;
    RespBody::from_result(&user).response()
}

async fn send_to_user(
    username: &str,
    title: &str,
    content: &str,
    environment: &ApplicationEnvironment,
) -> Result<String, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let user_service = application_context.context.get::<UserService>();
    let _user = user_service.get_user(&username).await?;

    let app_id = environment.get_property::<String>("wxcorp_app_id").unwrap();
    let domain = environment.get_property::<String>("server.domain").unwrap();
    let result = send_by_wx_corp(&domain, app_id.as_str(), &username, title, content).await;
    match result {
        Ok(s) => Ok(s),
        Err(e) => Err(e.into()),
    }
}

#[derive(Template)]
#[template(path = "message.html")]
struct MessageTemplate {
    title: String,
    content: String,
    send_time: rbatis::rbdc::DateTime,
}

pub async fn handle_message_detail(Path(id): Path<String>) -> impl IntoResponse {
    let message = message_svc::get_message_detail(&id).await;
    let message = message.unwrap();
    let template = MessageTemplate {
        title: message.title.unwrap(),
        content: message.content.unwrap(),
        send_time: message.send_time.unwrap(),
    };
    HtmlTemplate(template)
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

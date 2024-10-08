use application_beans::factory::bean_factory::BeanFactory;
use application_boot::application::APPLICATION_CONTEXT;
use application_core::env::environment::ApplicationEnvironment;
use application_core::env::property_resolver::PropertyResolver;
use application_web::response::RespBody;
use application_web_macros::{get, post};
use askama::Template;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use tracing::debug;

use crate::service::auth::{self, Signature};
use crate::service::message::{get_message_detail, save_message_record, send_by_wx_corp};
use crate::service::user::UserService;
use crate::service::wx_corp;

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

#[get("/")]
pub async fn do_get_wx_corp_receive(Query(params): Query<WxCorpJoinValidate>) -> impl IntoResponse {
    debug!("get /");
    debug!("query params: {:?}", params);

    let msg_signature = &params.msg_signature;
    let time_stamp = &params.timestamp;
    let nonce = &params.nonce;
    let echo_str = &params.echo_str;
    let application_context = APPLICATION_CONTEXT.read().await;
    let environment = application_context.get_environment().await;
    let token = environment.get_property::<String>("wxcorp.token").unwrap();
    let aes_key = environment
        .get_property::<String>("wxcorp.encoding_aes_key")
        .unwrap();
    let result = wx_corp::verify_url(
        msg_signature,
        token.as_str(),
        time_stamp,
        nonce,
        echo_str,
        aes_key.as_str(),
    );
    RespBody::result(&result).response()
}

#[post("/")]
pub async fn do_post_wx_corp_receive() -> impl IntoResponse {
    RespBody::<()>::success_info("").response()
}

#[post("/send/:username")]
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
    let environment = application_context.get_environment().await;
    let token = environment.get_property::<String>("wxcorp.token").unwrap();
    if !auth::check_signature(signature, token.as_str(), timestamp, nonce, content) {
        debug!("auth failed!");
        return (StatusCode::FORBIDDEN, "auth failed").into_response();
    }
    debug!("auth pass!");
    debug!("msg:{}", message.content);

    let title = message.title.clone().unwrap_or_default();
    let user = send_to_user(&username, &title, content, &environment).await;
    RespBody::result(&user).response()
}

async fn send_to_user(
    username: &str,
    title: &str,
    content: &str,
    environment: &ApplicationEnvironment,
) -> Result<String, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let user_service = application_context.get_bean_factory().get::<UserService>();
    let _user = user_service.get_user(username).await?;

    let app_id = environment.get_property::<String>("wxcorp.app_id").unwrap();
    let domain = environment.get_property::<String>("server.domain").unwrap();
    let message_uuid = uuid::Uuid::new_v4().to_string();
    let result = send_by_wx_corp(
        &domain,
        app_id.as_str(),
        username,
        &message_uuid,
        title,
        content,
    )
    .await?;
    save_message_record(&message_uuid, title, content, username).await?;
    Ok(result)
}

#[derive(Template)]
#[template(path = "message.html")]
struct MessageTemplate {
    title: String,
    content: String,
    send_time: NaiveDateTime,
}

#[get("/message/:id")]
pub async fn handle_message_detail(Path(id): Path<String>) -> impl IntoResponse {
    let message = get_message_detail(&id).await.unwrap();
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

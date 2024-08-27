use std::error::Error;

use application::application::APPLICATION_CONTEXT;
use chrono::Local;
use database_mysql_seaorm::Dao;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde_derive::{Deserialize, Serialize};

use crate::entity::message_receiver;
use crate::service::wx_corp::WxCorpService;
use crate::entity::message;

#[derive(Debug, Serialize, Deserialize)]
pub struct TextCardMessage {
    pub touser: Option<String>,
    pub toparty: Option<String>,
    pub totag: Option<String>,
    pub msgtype: String,
    pub agentid: String,
    pub textcard: TextCard,
    pub enable_id_trans: i32,
    pub enable_duplicate_check: i32,
    pub duplicate_check_interval: i32,
    pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextCard {
    pub title: String,
    pub description: String,
    pub url: String,
    pub btntxt: String,
}

impl TextCardMessage {
    pub fn new(
        domain: &str,
        app_id: &str,
        user: &str,
        uuid: &str,
        title: &str,
        content: &str,
    ) -> Self {
        TextCardMessage {
            uuid: Some(uuid.to_string()),
            touser: Some(user.to_string()),
            toparty: None,
            totag: None,
            msgtype: "textcard".to_string(),
            agentid: app_id.to_string(),
            textcard: TextCard {
                title: title.to_string(),
                description: format!(
                    "<div class=\"normal\">{}</div><div class=\"gray\">通知时间：{}</div>",
                    content.to_string(),
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
                url: format!("https://{}/message/{}", domain, uuid),
                btntxt: "查看详情".to_string(),
            },
            enable_id_trans: 0,
            enable_duplicate_check: 0,
            duplicate_check_interval: 0,
        }
    }
}

pub async fn send_by_wx_corp(
    domain: &str,
    app_id: &str,
    username: &str,
    uuid: &str,
    title: &str,
    content: &str,
) -> Result<String, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let msg = TextCardMessage::new(domain, app_id, username, uuid, title, content);
    let json = serde_json::to_string(&msg).unwrap();
    let result = application_context
        .context
        .get::<WxCorpService>()
        .send(&json)
        .await;

    match result {
        Ok(result) => {
            let response = serde_json::to_string(&result)?;
            Ok(response)
        }
        Err(e) => {Err(e.into())}
    }
}

pub async fn save_message_record(
    message_uuid: &str,
    title: &str,
    content: &str,
    username: &str,
) -> Result<(), Box<dyn Error>> {
    let message = message::ActiveModel {
        id: NotSet,
        uuid: Set(Some(message_uuid.to_string())),
        title: Set(Some(title.to_string())),
        content: Set(Some(content.to_string())),
        send_time: Set(Some(Local::now().naive_local())),
    };
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.context.get::<Dao>();
    let conn = &dao.connection;
    let tx = conn.begin().await?;
    let result = message::Entity::insert(message).exec(conn).await?;
    let message_id = result.last_insert_id;
    let message_receiver = message_receiver::ActiveModel {
        id: NotSet,
        message_id: Set(Some(message_id)),
        user_id: Set(Some(username.to_string())),
    };
    message_receiver.save(conn).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_message_detail(uuid: &str) -> Result<Option<message::Model>, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.context.get::<Dao>();
    let message = message::Entity::find()
        .filter(message::Column::Uuid.eq(uuid))
        .one(&dao.connection)
        .await?;
    Ok(message)
}

use std::error::Error;
use std::ops::Not;

use application::application::APPLICATION_CONTEXT;
use database_mysql_seaorm::Dao;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, Set, TransactionTrait};
use serde_derive::{Deserialize, Serialize};

use crate::{
    entity::message,
    wx_corp::{SendMessageResult, WxCorpService},
};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub send_time: Option<DateTime>,
}
crud!(Message {});
impl_select!(Message{select_by_uuid(uuid:&str) -> Option => "`where uuid = #{uuid}`"});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageReceiver {
    pub id: Option<i64>,
    pub message_id: Option<u64>,
    pub user_id: Option<String>,
}
crud!(MessageReceiver {});

impl TextCardMessage {
    pub fn new(domain: &str, app_id: &str, user: &str, message: &Message) -> Self {
        TextCardMessage {
            uuid: message.uuid.clone(),
            touser: Some(user.to_string()),
            toparty: None,
            totag: None,
            msgtype: "textcard".to_string(),
            agentid: app_id.to_string(),
            textcard: TextCard {
                title: message.title.clone().unwrap_or_default(),
                description: format!(
                    "<div class=\"normal\">{}</div><div class=\"gray\">通知时间：{}</div>",
                    message.content.clone().unwrap(),
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
                url: format!(
                    "https://{}/message/{}",
                    domain,
                    message.uuid.clone().unwrap()
                ),
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
    title: &str,
    msg: &str,
) -> Result<String, Box<dyn Error>> {
    let message = message::ActiveModel {
        id: NotSet,
        uuid: Set(Some(uuid::Uuid::new_v4().to_string())),
        title: Set(Some(title.to_string())),
        content: Set(Some(msg.to_string())),
        send_time: Some(DateTime::now()),
    };
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.context.get::<Dao>();
    let conn = dao.connection;
    let tx = conn.begin().await?;
    message.save(&tx).await?;
    let message_id = tx.last_insert_id.as_u64().unwrap();
    let message_receiver = MessageReceiver {
        id: None,
        message_id: Some(message_id),
        user_id: Some(username.to_string()),
    };
    MessageReceiver::insert(rb, &message_receiver).await?;

    let mut response = SendMessageResult {
        errcode: -1,
        errmsg: "消息发送失败".to_string(),
        invaliduser: None,
        invalidparty: None,
        invalidtag: None,
        unlicenseduser: None,
        msgid: None,
        response_code: None,
    };

    let msg = TextCardMessage::new(domain, app_id, username, &message);
    let json = serde_json::to_string(&msg).unwrap();
    let result = application_context
        .context
        .get::<WxCorpService>()
        .send(&json)
        .await;

    match result {
        Ok(result) => {
            response = result;
            if response.is_success().not() {
                MessageReceiver::delete_by_column(rb, "message_id", message_id).await?;
                Message::delete_by_column(rb, "uuid", &message.uuid.unwrap()).await?;
            }
        }
        Err(_) => {
            MessageReceiver::delete_by_column(rb, "message_id", message_id).await?;
            Message::delete_by_column(rb, "uuid", &message.uuid.unwrap()).await?;
        }
    }
    // tx.commit().await?;
    // tx.rollback().await?;
    let response = serde_json::to_string(&response)?;
    Ok(response)
}

pub async fn get_message_detail(uuid: &str) -> Option<Message> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let rb = application_context.context.get::<DbService>().dao();
    Message::select_by_uuid(rb, uuid).await.unwrap()
}

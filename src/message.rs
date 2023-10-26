use rbatis::rbdc::datetime::DateTime;
use rbatis::{crud, impl_select};
use serde_derive::{Deserialize, Serialize};
use std::ops::Not;
use database::DatabaseService;

use crate::services::SERVICES;
use crate::wx_corp::WxCorpService;

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
    pub fn new(app_id: &str, user: &str, message: &Message) -> Self {
        TextCardMessage {
            uuid: message.uuid.clone(),
            touser: Some(user.to_string()),
            toparty: None,
            totag: None,
            msgtype: "textcard".to_string(),
            agentid: app_id.to_string(),
            textcard: TextCard {
                title: "设备通知".to_string(),
                description: format!("<div class=\"normal\">通知内容: {}</div><div class=\"normal\">{}</div><div class=\"gray\">通知时间：{}</div>",
                                     message.title.clone().unwrap_or_default(),
                                     message.content.clone().unwrap(),
                                     chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                ),
                url: format!("https://message.junhuitsai.space/message/{}", message.uuid.clone().unwrap()),
                btntxt: "查看详情".to_string(),
            },
            enable_id_trans: 0,
            enable_duplicate_check: 0,
            duplicate_check_interval: 0,
        }
    }
}

pub async fn send_by_wx_corp(app_id: &str, username: &str, title: &str,msg: &str) -> String {
    let message = Message {
        id: None,
        uuid: Some(uuid::Uuid::new_v4().to_string()),
        title: Some(title.to_string()),
        content: Some(msg.to_string()),
        send_time: Some(DateTime::now()),
    };
    let rb = SERVICES.get::<DatabaseService>().dao();
    let mut tx = rb.acquire_begin().await.unwrap();
    let result = Message::insert(&tx, &message).await.unwrap();
    let message_id = result.last_insert_id.as_u64().unwrap();
    let message_receiver = MessageReceiver {
        id: None,
        message_id: Some(message_id),
        user_id: Some(username.to_string()),
    };
    MessageReceiver::insert(&tx, &message_receiver)
        .await
        .unwrap();

    let msg = TextCardMessage::new(app_id, username, &message);

    let json = serde_json::to_string(&msg).unwrap();
    let result = SERVICES.get::<WxCorpService>().send(&json).await;
    if result.is_success().not() {
        MessageReceiver::delete_by_column(&tx, "message_id", message_id)
            .await
            .unwrap();
        Message::delete_by_column(&tx, "uuid", &message.uuid.unwrap())
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    tx.rollback().await.unwrap();
    serde_json::to_string(&result).unwrap()
}

pub async fn get_message_detail(uuid: &str) -> Option<Message> {
    let rb = SERVICES.get::<DatabaseService>().dao();
    let message = Message::select_by_uuid(&rb, uuid).await.unwrap();
    message
}

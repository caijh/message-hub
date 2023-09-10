use serde_derive::{Deserialize, Serialize};

use crate::config::CONFIG;

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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextCard {
    pub title: String,
    pub description: String,
    pub url: String,
    pub btntxt: String,
}


pub fn parse_message(username: &str, msg: &String) -> TextCardMessage {
    let agentid: String = CONFIG.wxcorp_app_id.clone();
    TextCardMessage {
        touser: Some(username.to_string()),
        toparty: None,
        totag: None,
        msgtype: "textcard".to_string(),
        agentid,
        textcard: TextCard {
            title: "设备通知".to_string(),
            description: msg.to_string(),
            url: "".to_string(),
            btntxt: "更多".to_string(),
        },
        enable_id_trans: 0,
        enable_duplicate_check: 0,
        duplicate_check_interval: 0,
    }
}


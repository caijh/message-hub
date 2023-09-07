use serde_derive::{Deserialize, Serialize};
use crate::config::CONFIG;
use lazy_static::lazy_static;
use crate::auth::AccessToken;

#[derive(Debug, Serialize, Deserialize)]
struct GetTokenResult {
    errcode: i32,
    errmsg: String,
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResult {
    errcode: i32,
    errmsg: String,
    invaliduser: Option<String>,
    invalidparty: Option<String>,
    invalidtag: Option<String>,
    unlicenseduser: Option<String>,
    msgid: Option<String>,
    response_code: Option<String>
}



pub struct WxCorpInterface {
    storage: super::storage::SingleKvStorage,
}

const STORE: &str = "wxcorp";

impl Default for WxCorpInterface {
    fn default() -> Self {
        WxCorpInterface::new()
    }
}

impl WxCorpInterface {
    pub fn new() -> WxCorpInterface {
        WxCorpInterface {
            storage: super::storage::SingleKvStorage::new(&CONFIG.db_path, STORE),
        }
    }

    fn get_access_token_internal(&self) -> AccessToken {
        let config = CONFIG.clone();
        let corpid = config.corpid;
        let secret = config.corpsecret;
        let client = reqwest::Client::new();
        let res: GetTokenResult = client
            .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
            .query(&[("corpid", &corpid)])
            .query(&[("corpsecret", &secret)])
            .send()
            .unwrap()
            .json()
            .unwrap();
        let expires = chrono::Utc::now() + chrono::Duration::seconds(res.expires_in);
        let token = res.access_token;
        AccessToken {
            access_token: token,
            expires: expires.timestamp(),
        }
    }

    pub fn get_access_token(&self) -> AccessToken {
        // 尝试从数据库获取access token
        let token = self.storage.get_single("access_token");
        match token {
            Some(token_string) => {
                let access_token: AccessToken = serde_json::from_str(&token_string).unwrap();
                let now = chrono::Utc::now();
                // 过期重新获取
                if access_token.expires <= now.timestamp() {
                    self.update_access_token()
                } else {
                    access_token
                }
            }
            None => self.update_access_token(),
        }
    }

    fn update_access_token(&self) -> AccessToken {
        let new_token = self.get_access_token_internal();
        let json_string = serde_json::to_string(&new_token).unwrap();
        self.storage
            .put_single("access_token", &rkv::Value::Json(&json_string));
        new_token
    }

    pub fn send(&self, body: &str) -> SendMessageResult {
        let access_token = self.get_access_token().access_token;
        let client = reqwest::Client::new();
        let res: SendMessageResult = client
            .post("https://qyapi.weixin.qq.com/cgi-bin/message/send")
            .query(&[("access_token", &access_token)])
            .body(body.to_string())
            .send()
            .unwrap()
            .json()
            .unwrap();
        res
    }
}

lazy_static! {
    pub static ref INTERFACE: WxCorpInterface = WxCorpInterface::new();
}

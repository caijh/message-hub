use config::Config;
use serde_derive::{Deserialize, Serialize};

use crate::services::SERVICES;
use crate::wx_corp::WxCorpService;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct User {
    pub errcode: i32,
    pub errmsg: String,
    pub userid: Option<String>,
    pub name: Option<String>,
    pub department: Option<Vec<String>>,
    pub order: Option<Vec<String>>,
    pub position: Option<String>,
    pub mobile: Option<String>,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub biz_mail: Option<String>,
    pub is_leader_in_dept: Option<Vec<String>>,
    pub direct_leader: Option<Vec<i32>>,
    pub avatar: Option<String>,
    pub thumb_avatar: Option<String>,
    pub telephone: Option<String>,
    pub alias: Option<String>,
    pub status: Option<i32>,
    pub address: Option<String>,
    pub open_userid: Option<String>,
    pub main_department: Option<String>,
}

pub struct UserService {
    storage: super::storage::SingleKvStorage,
}

const STORE: &str = "user";

impl UserService {
    pub fn new(config: &Config) -> UserService {
        UserService {
            storage: super::storage::SingleKvStorage::new(config.get_string("db_path").unwrap().as_str(), STORE),
        }
    }

    async fn get_user_name_internal(&self, id: &str) -> Result<User, reqwest::Error> {
        let wx_corp_service = SERVICES.get::<WxCorpService>();
        let token = wx_corp_service.get_access_token().await;
        let client = reqwest::Client::new();
        let res:Result<User, reqwest::Error> = client
            .get("https://qyapi.weixin.qq.com/cgi-bin/user/get")
            .query(&[("access_token", token.access_token.as_str())])
            .query(&[("userid", id)])
            .send().await.unwrap()
            .json().await;
        res
    }

    pub async fn get_user(&self, id: &str) -> Result<User, &str> {
        let user = self.storage.get_single(id);
        match user {
            Some(user_string) => {
                let _user: User = serde_json::from_str(&user_string).unwrap();
                Ok(_user)
            }
            None => {
                let r = self.get_user_name_internal(id).await;
                match r {
                    Ok(u) => {
                        if u.errcode != 0 {
                            return Err("未找到用户");
                        }
                        self.storage.put_single(id, &rkv::Value::Json(&serde_json::to_string(&u).unwrap()));
                        Ok(u)
                    }
                    Err(_) => Err("未找到用户")
                }
            }
        }
    }
}

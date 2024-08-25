use std::error::Error;

use application::application::APPLICATION_CONTEXT;
use redis::Commands;
use redis_io::Redis;
use serde_derive::{Deserialize, Serialize};

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

#[derive(Default)]
pub struct UserService {}

impl UserService {
    async fn get_user_name_internal(&self, id: &str) -> Result<User, Box<dyn Error>> {
        let application_context = APPLICATION_CONTEXT.read().await;
        let wx_corp_service = application_context.context.get::<WxCorpService>();
        let token = wx_corp_service.get_access_token().await?;
        let client = reqwest::Client::new();
        let res: Result<User, reqwest::Error> = client
            .get("https://qyapi.weixin.qq.com/cgi-bin/user/get")
            .query(&[("access_token", token.access_token.as_str())])
            .query(&[("userid", id)])
            .send()
            .await
            .unwrap()
            .json()
            .await;
        match res {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn get_user(&self, id: &str) -> Result<User, Box<dyn Error>> {
        let client = Redis::get_client();
        let mut con = client.get_connection()?;
        let key = "App:Message:U:".to_string() + id;
        let user = con.get::<&str, Option<String>>(&key)?;
        match user {
            None => {
                let u = self.get_user_name_internal(id).await?;
                if u.errcode != 0 {
                    return Err(u.errmsg.into());
                }
                con.set_ex::<&str, String, String>(&key, serde_json::to_string(&u).unwrap(), 1800)?;
                Ok(u)
            }
            Some(user) => {
                let _user: User = serde_json::from_str(&user).unwrap();
                Ok(_user)
            }
        }
    }
}

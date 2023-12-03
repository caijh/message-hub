use std::error::Error;

use aes::Aes256;
use aes::cipher::block_padding::Pkcs7;
use base64::{alphabet, Engine};
use base64::engine::GeneralPurpose;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;
use configuration::Configuration;
use redis::Commands;
use redis_util::Redis;
use serde_derive::{Deserialize, Serialize};
use sha1_smol::Sha1;

use crate::auth::AccessToken;

type AesCbcDec = Decryptor<Aes256>;

#[derive(Debug, Serialize, Deserialize)]
struct GetTokenResult {
    errcode: i32,
    errmsg: String,
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageResult {
    pub errcode: i32,
    pub errmsg: String,
    pub invaliduser: Option<String>,
    pub invalidparty: Option<String>,
    pub invalidtag: Option<String>,
    pub unlicenseduser: Option<String>,
    pub msgid: Option<String>,
    pub response_code: Option<String>,
}

impl SendMessageResult {
    pub fn is_success(&self) -> bool {
        self.errcode == 0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptMessage {
    pub content: String,
    pub from_receive_id: String,
}

#[derive(Default)]
pub struct WxCorpService {}


impl WxCorpService {
    async fn get_access_token_internal(&self) -> AccessToken {
        let config = Configuration::get_config().await;
        let corpid = config.get_string("wxcorp_id").unwrap();
        let secret = config.get_string("wxcorp_secret").unwrap();
        let client = reqwest::Client::new();
        let res: GetTokenResult = client
            .get("https://qyapi.weixin.qq.com/cgi-bin/gettoken")
            .query(&[("corpid", &corpid)])
            .query(&[("corpsecret", &secret)])
            .send().await.unwrap()
            .json().await.unwrap();
        let expires = chrono::Utc::now() + chrono::Duration::seconds(res.expires_in);
        let token = res.access_token;
        AccessToken {
            access_token: token,
            expires: expires.timestamp(),
        }
    }

    pub async fn get_access_token(&self) -> Result<AccessToken, Box<dyn Error>> {
        // 尝试从数据库获取access token
        let client = Redis::get_redis_client();
        let mut con = client.get_connection()?;
        let key = "App:MessageHub:AccessToken:WxCorp";
        let token = con.get::<&str, Option<String>>(key)?;
        match token {
            None => {
                let access_token = self.update_access_token().await;
                Ok(access_token)
            }
            Some(token) => {
                let access_token: AccessToken = serde_json::from_str(&token).unwrap();
                let now = chrono::Utc::now();
                // 过期重新获取
                let access_token = if access_token.expires <= now.timestamp() {
                    self.update_access_token().await
                } else {
                    access_token
                };
                Ok(access_token)
            }
        }
    }

    async fn update_access_token(&self) -> AccessToken {
        let new_token = self.get_access_token_internal().await;
        let json_string = serde_json::to_string(&new_token).unwrap();
        let client = Redis::get_redis_client();
        let mut con = client.get_connection().expect("");
        let key = "App:MessageHub:AccessToken:WxCorp";
        con.set_ex::<&str, String, String>(key, json_string, 60 * 60 * 24).expect("Fail to update access token");
        new_token
    }

    pub async fn send(&self, body: &str) -> Result<SendMessageResult, Box<dyn Error>> {
        let access_token = self.get_access_token().await?.access_token;
        let client = reqwest::Client::new();
        let result = client
            .post("https://qyapi.weixin.qq.com/cgi-bin/message/send")
            .query(&[("access_token", &access_token)])
            .body(body.to_string())
            .send().await;
        match result {
            Ok(resp) => {
                let json: SendMessageResult = resp.json().await.unwrap();
                Ok(json)
            }
            Err(e) => Err(e.into())
        }
    }
}

pub fn verify_url(
    msg_signature: &str,
    token: &str,
    time_stamp: &str,
    nonce: &str,
    echo_str: &str,
    aes_key: &str,
) -> Result<String, String> {
    // 校验签名
    let mut args: Vec<&str> = vec![token, time_stamp, nonce, echo_str];
    args.sort();
    let message = args.join("");
    let mut hasher = Sha1::default();
    hasher.update(message.as_bytes());
    let signature = hasher.digest().to_string();
    tracing::info!("Calculated signature: {}", signature);
    if signature != msg_signature {
        return Err("AesException.ValidateSignatureError".to_string());
    }

    let result = decrypt(aes_key, echo_str);

    String::from_utf8(result.content.into_bytes()).map_err(|_| "Utf8DecodingError".to_string())
}

pub fn decrypt(aes_key: &str, text: &str) -> DecryptMessage {
    // decode key from base64 aes_key and create cipher.
    let engine = GeneralPurpose::new(&alphabet::STANDARD, base64::engine::general_purpose::NO_PAD);
    let aes_key_bin = engine.decode(aes_key).unwrap();
    let iv = &aes_key_bin[..16];
    let cipher = AesCbcDec::new_from_slices(aes_key_bin.as_slice(), iv).unwrap();

    // decode from base64 text.
    let engine = GeneralPurpose::new(&alphabet::STANDARD, base64::engine::general_purpose::PAD);
    let encrypted = engine.decode(text).unwrap();

    // use cipher to decrypt.
    let decrypted = cipher.decrypt_padded_vec_mut::<Pkcs7>(&encrypted).unwrap();

    let content = &decrypted[16..];
    let length = u32::from_be_bytes(content[0..4].try_into().unwrap()) as usize;
    let msg_content = String::from_utf8_lossy(&content[4..(4 + length)]).to_string();
    let receive_id = String::from_utf8_lossy(&content[(4 + length)..]).to_string();

    DecryptMessage {
        content: msg_content,
        from_receive_id: receive_id,
    }
}


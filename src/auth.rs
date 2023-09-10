use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use log::debug;
use serde_derive::{Deserialize, Serialize};

use crate::config::CONFIG;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct AccessToken {
    pub access_token: String,
    pub expires: i64, // 过期时间
}

#[derive(Deserialize, Debug)]
pub struct Signature {
    pub signature: String,
    pub timestamp: String,
    pub nonce: String,
}

pub fn check_signature(signature: &str, timestamp: &str, nonce: &str, content: &str) -> bool {
    debug!("signature:{}", signature);
    debug!("timestamp:{}", timestamp);
    debug!("nonce:{}", nonce);
    let token: String = CONFIG.wxcorp_token.clone();
    let content = STANDARD.decode(content.as_bytes()).unwrap();
    let content = String::from_utf8(content).unwrap();
    let mut v = [token, timestamp.to_string(), nonce.to_string(), content];
    v.sort();

    let mut hasher = Sha1::new();
    hasher.input_str(format!("{}{}{}", v[0], v[1], v[2]).as_str());

    let hex = hasher.result_str();
    debug!("Calculated signature:{}", hex);
    hex == signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_signature_positive() {
        let signature = "c9b76d0a81c77874773537e40239809762e1166e";
        let timestamp = "1234567890";
        let nonce = "xyz";
        let content = "content";
        let result = check_signature(signature, timestamp, nonce, content);
        assert_eq!(result, true);
    }

    #[test]
    fn test_check_signature_negative() {
        let signature = "aabbcc";
        let timestamp = "1234567890";
        let nonce = "xyz";
        let content = "content";
        let result = check_signature(signature, timestamp, nonce, content);
        assert_eq!(result, false);
    }
}

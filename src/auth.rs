use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use log::debug;
use serde_derive::{Deserialize, Serialize};

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


pub fn get_signature(token: &str, timestamp: &str, nonce: &str, content: &str) -> String {
    let content = STANDARD.encode(content.as_bytes()); // get base64 string of content
    let mut v = [token, timestamp, nonce, content.as_str()];
    v.sort();

    let mut hasher = Sha1::new();
    hasher.input_str(format!("{}{}{}{}", v[0], v[1], v[2], v[3]).as_str());

    hasher.result_str()
}

pub fn check_signature(signature: &str, token: &str, timestamp: &str, nonce: &str, content: &str) -> bool {
    debug!("signature:{}", signature);
    debug!("timestamp:{}", timestamp);
    debug!("nonce:{}", nonce);
    let hex = get_signature(token, timestamp, nonce, content);
    debug!("Calculated signature: {}", hex);
    hex == signature
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_signature_positive() {
        let timestamp = "1234567890";
        let nonce = "xyz";
        let content = "content";
        let token = "token";
        let signature = get_signature(token, timestamp, nonce, content);
        let result = check_signature(&signature, token, timestamp, nonce, content);
        assert_eq!(result, true);
    }
}

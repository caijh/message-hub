use crate::entity::user_token;
use application_beans::factory::bean_factory::BeanFactory;
use application_context::context::application_context::APPLICATION_CONTEXT;
use database_mysql_seaorm::Dao;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct AccessToken {
    pub access_token: String,
    pub expires: i64, // 过期时间
}

pub async fn get_user_token(token: &str) -> Result<Option<user_token::Model>, Box<dyn Error>> {
    let application_context = APPLICATION_CONTEXT.read().await;
    let dao = application_context.get_bean_factory().get::<Dao>();
    let conn = &dao.connection;
    let user_token = user_token::Entity::find()
        .filter(user_token::Column::Token.eq(token))
        .one(conn)
        .await?;

    Ok(user_token)
}

pub async fn check_token(token: &str) -> (bool, String) {
    let user_token = get_user_token(token).await;
    match user_token {
        Ok(user_token) => {
            if user_token.is_none() {
                (false, "auth failed".to_string())
            } else {
                let user_token = user_token.unwrap();
                (true, user_token.username)
            }
        }
        Err(_) => (false, "auth failed".to_string()),
    }
}

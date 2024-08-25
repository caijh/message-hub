use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "message")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub uuid: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub send_time: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

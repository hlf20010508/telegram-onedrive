/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::{
    entity::prelude::DeriveEntityModel, ActiveModelBehavior, DerivePrimaryKey, DeriveRelation,
    EntityTrait, EnumIter, PrimaryKeyTrait, Related, RelationDef, RelationTrait,
};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub username: String,
    pub expiration_timestamp: i64,
    pub access_token: String,
    pub refresh_token: String,
    pub root_path: String,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::current_user::Entity")]
    CurrentUser,
}

impl Related<Self> for Entity {
    fn to() -> RelationDef {
        Relation::CurrentUser.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

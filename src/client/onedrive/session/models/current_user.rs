/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use sea_orm::entity::prelude::DeriveEntityModel;
use sea_orm::{
    ActiveModelBehavior, DerivePrimaryKey, DeriveRelation, EntityTrait, EnumIter, PrimaryKeyTrait,
    Related, RelationDef, RelationTrait,
};

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "current_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i64,
}

#[derive(Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::session::Entity",
        from = "Column::UserId",
        to = "super::session::Column::Id"
    )]
    Session,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

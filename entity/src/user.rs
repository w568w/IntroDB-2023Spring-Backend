use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Sex;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    // 身份验证
    // - 密码使用 bcrypt 算法加盐储存
    pub password_salt: String,
    // - JWToken
    pub jwt_refresh_token: String,
    pub jwt_access_token: String,
    // 个人信息
    pub name: String,
    pub role: String,
    pub real_name: String,
    pub sex: Sex,
    // - TODO: 绩效指标
    #[sea_orm(ignore)]
    pub kpi: (),
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::order_list::Entity")]
    OrderList,
}

impl Related<super::order_list::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderList.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

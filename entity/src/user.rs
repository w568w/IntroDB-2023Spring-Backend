use crate::{to_active, Sex};
use fromsuper::FromSuper;
use redis_macros::{FromRedisValue, ToRedisArgs};
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, IntoActiveModel};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, FromRedisValue, ToRedisArgs,
)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    // 身份验证
    // - 密码使用 bcrypt 算法加盐储存
    pub password_salt: String,
    // - JWToken Key
    pub secret_key: String,
    // 个人信息
    pub role: String,
    pub real_name: String,
    pub sex: Sex,
    /// 是否已经删除
    #[sea_orm(default_value = "false")]
    pub is_deleted: bool,
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

#[derive(FromSuper, ToSchema, Serialize)]
#[fromsuper(from_type = "Model")]
pub struct GetUser {
    pub id: i32,
    pub role: String,
    pub real_name: String,
    pub sex: Sex,
}

#[derive(ToSchema, DeriveIntoActiveModel, Deserialize)]
pub struct NewUser {
    pub password_salt: String,
    pub role: String,
    pub real_name: String,
    pub sex: Sex,
}

#[derive(ToSchema, Deserialize)]
pub struct UpdateUser {
    pub password_salt: Option<String>,
    pub role: Option<String>,
    pub real_name: Option<String>,
    pub sex: Option<Sex>,
}

impl IntoActiveModel<ActiveModel> for UpdateUser {
    fn into_active_model(self) -> ActiveModel {
        ActiveModel {
            id: NotSet,
            password_salt: to_active(self.password_salt),
            secret_key: NotSet,
            role: to_active(self.role),
            real_name: to_active(self.real_name),
            sex: to_active(self.sex),
            is_deleted: NotSet,
        }
    }
}

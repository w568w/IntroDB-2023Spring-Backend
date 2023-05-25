use fromsuper::FromSuper;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "book")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub isbn: String,
    // 书籍信息
    pub title: String,
    pub author: String,
    pub publisher: String,
    // - 建议售价
    pub out_price: f32,
    // 存货信息
    // - 库存（但未上架）数量
    pub inventory_count: i32,
    // - 正在架上的数量
    pub on_shelf_count: i32,
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

#[derive(ToSchema, DeriveIntoActiveModel, Deserialize)]
pub struct UpdateBook {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub out_price: f32,
}

#[derive(Clone, ToSchema, DeriveIntoActiveModel, Serialize, Deserialize, FromSuper)]
#[fromsuper(from_type = "Model")]
pub struct NewBookInfo {
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub out_price: f32,
}

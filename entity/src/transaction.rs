use fromsuper::FromSuper;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "transaction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    // 交易信息
    // - 创建时间
    pub created_at: DateTime,
    // - 总价格（正数为收入，负数为支出）
    pub total_price: f32,
    // 外键连接
    // - OrderList
    pub ticket_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::order_list::Entity",
    from = "Column::TicketId",
    to = "super::order_list::Column::Id",
    )]
    OrderList,
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(FromSuper, ToSchema, Serialize)]
#[fromsuper(from_type = "Model")]
pub struct GetTransaction {
    pub id: i64,
    pub created_at: DateTime,
    pub total_price: f32,
    pub ticket_id: i32,
}
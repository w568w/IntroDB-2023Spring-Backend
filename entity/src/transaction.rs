use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
    // - SoldList/StockinList
    pub ticket_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}

impl ActiveModelBehavior for ActiveModel {}
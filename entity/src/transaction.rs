use chrono::Utc;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{order_list, TicketType};

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
        to = "super::order_list::Column::Id"
    )]
    OrderList,
}

impl Related<super::order_list::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrderList.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<order_list::Model> for ActiveModel {
    fn from(order: order_list::Model) -> Self {
        Self {
            id: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            total_price: Set(order.total_price),
            ticket_id: Set(order.id),
        }
    }
}

#[derive(ToSchema, Serialize)]
pub struct GetTransaction {
    pub id: i64,
    pub created_at: DateTime,
    pub total_price: f32,
    pub ticket_id: i32,
}

impl From<(Model, Option<crate::order_list::Model>)> for GetTransaction {
    fn from((transaction, order): (Model, Option<crate::order_list::Model>)) -> Self {
        Self {
            id: transaction.id,
            created_at: transaction.created_at,
            total_price: match order.map(|m| m.typ) {
                Some(TicketType::Sell) => transaction.total_price,
                Some(TicketType::Stock) => -transaction.total_price,
                None => transaction.total_price,
            },
            ticket_id: transaction.ticket_id,
        }
    }
}

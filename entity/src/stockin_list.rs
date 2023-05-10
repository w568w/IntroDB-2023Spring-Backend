use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::TicketStatus;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stockin_list")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    // 订单信息
    // - 需要支付的总价格
    pub in_price: f32,
    // - 进货的总数量
    pub total_count: i32,
    // 订单状态
    pub status: TicketStatus,
    // 订单元信息
    // - 创建时间
    pub created_at: DateTime,
    // - 更新时间
    pub updated_at: DateTime,
    // 外键连接
    // - Book
    pub book_isbn: String,
    // - User
    pub operator_id: i32,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
    belongs_to = "super::book::Entity",
    from = "Column::BookIsbn",
    to = "super::book::Column::Isbn",
    )]
    Book,
    #[sea_orm(
    belongs_to = "super::user::Entity",
    from = "Column::OperatorId",
    to = "super::user::Column::Id",
    )]
    User,
}

impl Related<super::book::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Book.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
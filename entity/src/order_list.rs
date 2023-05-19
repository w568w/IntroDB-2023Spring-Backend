use crate::{book::NewBookInfo, TicketStatus};
use chrono::offset::Utc;
use fromsuper::FromSuper;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_list")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    // 订单信息
    // - 实际支付的总价格
    pub total_price: f32,
    // - 实际购买的总数量
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
        to = "super::book::Column::Isbn"
    )]
    Book,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OperatorId",
        to = "super::user::Column::Id"
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

#[derive(ToSchema, Deserialize)]
pub struct NewOrder {
    pub book_isbn: String,
    pub total_price: f32,
    pub total_count: i32,
    #[serde(flatten)]
    pub book: Option<NewBookInfo>,
}

impl NewOrder {
    pub fn into_active_model(self, operator_id: i32) -> ActiveModel {
        ActiveModel {
            id: NotSet,
            total_price: Set(self.total_price),
            total_count: Set(self.total_count),
            status: Set(TicketStatus::Pending),
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            book_isbn: Set(self.book_isbn),
            operator_id: Set(operator_id),
        }
    }
}

#[derive(FromSuper, ToSchema, Serialize)]
#[fromsuper(from_type = "Model")]
pub struct GetOrder {
    pub id: i32,
    pub total_price: f32,
    pub total_count: i32,
    pub status: TicketStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub book_isbn: String,
    pub operator_id: i32,
}
pub mod book;
pub mod order_list;
pub mod transaction;
pub mod user;

use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveValue};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn to_active<V: Into<Value>, T: IntoActiveValue<V>>(value: Option<T>) -> ActiveValue<V> {
    match value {
        Some(v) => v.into_active_value(),
        None => ActiveValue::NotSet,
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[non_exhaustive]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_status")]
pub enum TicketStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "StockPaid")]
    StockPaid,
    #[sea_orm(string_value = "Done")]
    Done,
    #[sea_orm(string_value = "Revoked")]
    Revoked,
}

impl Default for TicketStatus {
    fn default() -> Self {
        TicketStatus::Pending
    }
}

impl IntoActiveValue<TicketStatus> for TicketStatus {
    fn into_active_value(self) -> ActiveValue<TicketStatus> {
        ActiveValue::Set(self)
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[non_exhaustive]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_type")]
pub enum TicketType {
    #[sea_orm(string_value = "Sell")]
    Sell,
    #[sea_orm(string_value = "Stock")]
    Stock,
}

#[derive(
    Debug, PartialEq, Eq, Clone, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[non_exhaustive]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "sex")]
pub enum Sex {
    #[sea_orm(string_value = "Male")]
    Male,
    #[sea_orm(string_value = "Female")]
    Female,
    #[sea_orm(string_value = "NonBinary")]
    NonBinary,
}

impl IntoActiveValue<Sex> for Sex {
    fn into_active_value(self) -> ActiveValue<Sex> {
        ActiveValue::Set(self)
    }
}

pub mod book;
pub mod sold_list;
pub mod stockin_list;
pub mod transaction;
pub mod user;

use sea_orm::entity::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[non_exhaustive]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "ticket_status")]
pub enum TicketStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
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

#[derive(Debug, PartialEq, Eq, Clone, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
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

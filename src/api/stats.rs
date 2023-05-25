use std::borrow::Borrow;

use crate::contants;
use crate::utils::errors::internal_server_error;

use crate::utils::jwt::AllowAdmin;
use crate::utils::jwt::JwtClaims;
use crate::utils::permission::APermission;

use super::preclude::*;

use actix_web::web::Data;

use actix_web::{get, web::Query};

use entity::TicketStatus;
use entity::TicketType;

use entity::book::NewBookInfo;
use log::info;
use sea_orm::sea_query::types::Alias;
use sea_orm::sea_query::Expr;
use sea_orm::FromQueryResult;
use sea_orm::IntoSimpleExpr;
use sea_orm::Iterable;
use sea_orm::QueryOrder;
use sea_orm::Statement;

use sea_orm::ConnectionTrait;
use sea_orm::QuerySelect;
use sea_orm::QueryTrait;
use sea_orm::Select;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use serde::Serialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

const TOTAL_PRICE: &str = "tp";
const TOTAL_COUNT: &str = "tc";
const SINT_TYPE: &str = "signed integer";

#[derive(Deserialize, IntoParams)]
pub struct StatOption {
    pub span: StatSpan,
    pub all: Option<bool>,
}

impl StatOption {
    pub fn should_filter_user(&self, user: &entity::user::Model) -> bool {
        !self.all.unwrap_or(false) || user.role != contants::user_type::SUPER_ADMIN
    }

    pub fn with_constraint<E: EntityTrait>(
        &self,
        query: Select<E>,
        date_column: E::Column,
        user_column: E::Column,
        user: &entity::user::Model,
    ) -> Select<E> {
        let query = self.span.with_constraint(query, date_column);
        if self.should_filter_user(user) {
            query.filter(user_column.eq(user.id))
        } else {
            query
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum StatSpan {
    Day,
    Week,
    Month,
    All,
}

impl StatSpan {
    pub fn with_constraint<E: EntityTrait>(
        &self,
        query: Select<E>,
        column: E::Column,
    ) -> Select<E> {
        let days = match self {
            StatSpan::Day => 1,
            StatSpan::Week => 7,
            StatSpan::Month => 30,
            StatSpan::All => return query,
        };
        query.filter(column.gt(chrono::Utc::now().naive_utc() - chrono::Duration::days(days)))
    }
}

macro_rules! fn_select_ones {
    ($fn_name: ident; $($T: ident),*) => {
        async fn $fn_name<$($T: sea_orm::TryGetable),+>(
            db: impl Borrow<DatabaseConnection>,
            query: impl QueryTrait + QueryFilter,
            col_name: &[&str],
        ) -> AResult<($($T,)+)> {
            let count_result = db
                .borrow()
                .query_one(
                    query.build(db.borrow().get_database_backend()),
                )
                .await?
                .ok_or_else(|| internal_server_error("Unable to get count"))?;
            Ok(
                count_macro::count! {
                    ($(
                        count_result.try_get::<$T>("", col_name[_int_])?,
                    )+)
                }
            )
        }
    };
}

fn_select_ones!(select_one; T1);
fn_select_ones!(select_two; T1, T2);

#[derive(Serialize, ToSchema)]
pub struct StatTransaction {
    pub total_sell_price: f32,
    pub total_stock_paid_price: f32,
}

async fn stat_transaction_query(
    db: impl Borrow<DatabaseConnection>,
    typ: TicketType,
    base_query: impl QueryTrait + QueryFilter,
    col_name: &str,
) -> AResult<f32> {
    Ok(select_one::<Option<f32>>(
        db,
        base_query.filter(entity::order_list::Column::Typ.eq(typ)),
        &[col_name],
    )
    .await?
    .0
    .unwrap_or(0.0))
}

#[p(
    params(StatOption),
    responses(
        (status = OK, description = "Stat successful", body = StatTransaction),
    ),
    security(("jwt_token" = []))
)]
#[get("/stats/transaction")]
pub async fn stat_transaction(
    param: Query<StatOption>,
    db: Data<DatabaseConnection>,
    auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<StatTransaction>> {
    let query = param
        .span
        .with_constraint(
            entity::transaction::Entity::find(),
            entity::transaction::Column::CreatedAt,
        )
        .find_also_related(entity::order_list::Entity)
        .select_only()
        .column_as(entity::order_list::Column::TotalPrice.sum(), TOTAL_PRICE);

    let query = if param.should_filter_user(&auth.auth_info) {
        query.filter(entity::order_list::Column::OperatorId.eq(auth.auth_info.id))
    } else {
        query
    };
    Ok(AJson(StatTransaction {
        total_sell_price: stat_transaction_query(
            db.get_ref(),
            TicketType::Sell,
            query.clone(),
            TOTAL_PRICE,
        )
        .await?,
        total_stock_paid_price: stat_transaction_query(
            db.get_ref(),
            TicketType::Stock,
            query,
            TOTAL_PRICE,
        )
        .await?,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct StatStock {
    pub total_stock_count: i32,
    pub total_waiting_for_confirm_count: i32,
}

#[p(
    params(StatOption),
    responses(
        (status = OK, description = "Stat successful", body = StatStock),
    ),
    security(("jwt_token" = []))
)]
#[get("/stats/stock")]
pub async fn stat_stock(
    param: Query<StatOption>,
    db: Data<DatabaseConnection>,
    auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<StatStock>> {
    let query = param
        .with_constraint(
            entity::order_list::Entity::find(),
            entity::order_list::Column::CreatedAt,
            entity::order_list::Column::OperatorId,
            &auth.auth_info,
        )
        .filter(entity::order_list::Column::Typ.eq(TicketType::Stock))
        .select_only();

    Ok(AJson(StatStock {
        total_stock_count: select_one(
            db.get_ref(),
            query.clone().column_as(
                entity::order_list::Column::TotalCount
                    .sum()
                    .cast_as(Alias::new(SINT_TYPE)),
                TOTAL_COUNT,
            ),
            &[TOTAL_COUNT],
        )
        .await?
        .0,
        total_waiting_for_confirm_count: select_one(
            db.get_ref(),
            query
                .filter(entity::order_list::Column::Status.eq(TicketStatus::StockPaid))
                .column_as(
                    entity::order_list::Column::TotalCount
                        .sum()
                        .cast_as(Alias::new(SINT_TYPE)),
                    TOTAL_COUNT,
                ),
            &[TOTAL_COUNT],
        )
        .await?
        .0,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct StatSell {
    pub total_sell_count: i32,
    pub total_done_count: i32,
}

#[p(
    params(StatOption),
    responses(
        (status = OK, description = "Stat successful", body = StatSell),
    ),
    security(("jwt_token" = []))
)]
#[get("/stats/sell")]
pub async fn stat_sell(
    param: Query<StatOption>,
    db: Data<DatabaseConnection>,
    auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<StatSell>> {
    let query = param
        .with_constraint(
            entity::order_list::Entity::find(),
            entity::order_list::Column::CreatedAt,
            entity::order_list::Column::OperatorId,
            &auth.auth_info,
        )
        .filter(entity::order_list::Column::Typ.eq(TicketType::Sell))
        .select_only();

    Ok(AJson(StatSell {
        total_sell_count: select_one::<Option<i32>>(
            db.get_ref(),
            query.clone().column_as(
                entity::order_list::Column::TotalCount
                    .sum()
                    .cast_as(Alias::new(SINT_TYPE)),
                TOTAL_COUNT,
            ),
            &[TOTAL_COUNT],
        )
        .await?
        .0
        .unwrap_or(0),
        total_done_count: select_one::<Option<i32>>(
            db.get_ref(),
            query
                .filter(entity::order_list::Column::Status.eq(TicketStatus::Done))
                .column_as(
                    entity::order_list::Column::TotalCount
                        .sum()
                        .cast_as(Alias::new(SINT_TYPE)),
                    TOTAL_COUNT,
                ),
            &[TOTAL_COUNT],
        )
        .await?
        .0
        .unwrap_or(0),
    }))
}

#[derive(Serialize, ToSchema)]
pub struct StatBook {
    pub total_inventory_count: i32,
    pub total_book_count: i32,
}

#[p(
    responses(
        (status = OK, description = "Stat successful", body = StatBook),
    ),
    security(("jwt_token" = []))
)]
#[get("/stats/book")]
pub async fn stat_book(
    db: Data<DatabaseConnection>,
    _auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<StatBook>> {
    let query = entity::book::Entity::find().select_only();

    Ok(AJson(StatBook {
        total_inventory_count: select_one::<Option<i32>>(
            db.get_ref(),
            query.clone().column_as(
                entity::book::Column::InventoryCount
                    .sum()
                    .cast_as(Alias::new(SINT_TYPE)),
                TOTAL_COUNT,
            ),
            &[TOTAL_COUNT],
        )
        .await?
        .0
        .unwrap_or(0),
        total_book_count: select_one(
            db.get_ref(),
            query.column_as(
                entity::book::Column::Isbn
                    .count()
                    .cast_as(Alias::new(SINT_TYPE)),
                TOTAL_COUNT,
            ),
            &[TOTAL_COUNT],
        )
        .await?
        .0,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct StatBestsell {
    pub isbn: String,
    #[serde(flatten)]
    pub info: NewBookInfo,
    pub total_sell_count: i32,
}

#[p(
    params(StatOption),
    responses(
        (status = OK, description = "Stat successful", body = Vec<StatBestsell>),
    ),
    security(("jwt_token" = []))
)]
#[get("/stats/bestsell")]
pub async fn stat_bestsell(
    param: Query<StatOption>,
    db: Data<DatabaseConnection>,
    _auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<Vec<StatBestsell>>> {
    let mut query = param
        .span
        .with_constraint(
            entity::order_list::Entity::find(),
            entity::order_list::Column::CreatedAt,
        )
        .filter(entity::order_list::Column::Typ.eq(TicketType::Sell))
        .filter(entity::order_list::Column::Status.eq(TicketStatus::Done))
        .find_also_related(entity::book::Entity)
        .select_only()
        .columns(entity::book::Column::iter().collect::<Vec<_>>())
        .column_as(
            entity::order_list::Column::TotalCount
                .sum()
                .cast_as(Alias::new(SINT_TYPE)),
            TOTAL_COUNT,
        )
        .order_by_desc(Expr::val(TOTAL_COUNT))
        .limit(10);

    // query.group_by() 每次只能加一个，所以要加多个的话，需要用 add_group_by
    QueryOrder::query(&mut query).add_group_by(
        entity::book::Column::iter()
            .map(IntoSimpleExpr::into_simple_expr)
            .collect::<Vec<_>>(),
    );

    // fixme: 使用预烘焙的 SQL。为什么直接使用 query 会卡死？
    let baked_sql = query.build(db.get_database_backend()).to_string();
    info!("Query SQL: {}", baked_sql);
    let results = db
        .query_all(Statement::from_string(db.get_database_backend(), baked_sql))
        .await?;

    let mut books = Vec::with_capacity(results.len());

    for result in results {
        let book = entity::book::Model::from_query_result(&result, "")?;
        let count = result.try_get::<Option<i32>>("", TOTAL_COUNT)?.unwrap_or(0);
        books.push(StatBestsell {
            isbn: book.isbn.clone(),
            info: book.into(),
            total_sell_count: count,
        });
    }
    Ok(AJson(books))
}

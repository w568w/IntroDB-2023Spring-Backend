use crate::utils::errors::{conflict, not_found, unprocessable_entity};
use crate::utils::ext::SelectExt;
use crate::utils::jwt::{AllowAdmin, JwtClaims};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{
    get, post,
    web::{Path, Query},
};

use chrono::Utc;
use entity::order_list::{GetOrder, NewOrder};

use entity::{order_list, TicketStatus, TicketType};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, Set, TransactionTrait,
};

#[p(
    request_body = NewOrder,
    responses(
        (status = OK, description = "Book sold successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/sell")]
pub async fn sell_book(
    order: AJson<NewOrder>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    // 校验合法性
    if order.total_count <= 0 {
        return Err(unprocessable_entity("Total count must be positive").into());
    }
    // 校验书籍是否存在
    entity::book::Entity::find_by_id(&order.book_isbn)
        .one(db.get_ref())
        .await?
        .ok_or_else(|| not_found("Book not found"))?;
    // 创建订单
    Ok(AJson(
        order
            .into_inner()
            .into_active_model(auth.auth_info.id, TicketType::Sell)
            .insert(db.get_ref())
            .await?
            .into(),
    ))
}

#[p(
    params(PagingRequest),
    responses(
        (status = OK, description = "Get sell list successfully", body = [GetOrder]),
    ),
    security(("jwt_token" = []))
)]
#[get("/sell")]
pub async fn get_sell_list(
    paging: Query<PagingRequest>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<HttpResponse> {
    entity::order_list::Entity::find()
        .filter(order_list::Column::Typ.eq(TicketType::Sell))
        .paged::<DatabaseConnection, _, GetOrder>(paging.into_inner(), db.get_ref())
        .await
}

#[p(
    responses(
        (status = OK, description = "Customer buy book successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/sell/{id}/pay")]
pub async fn pay_sell(
    id: Path<i32>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let trans = db.begin().await?;
    // 修改订单（已完成）
    let order = pay_order(
        id.into_inner(),
        TicketStatus::Pending,
        TicketType::Sell,
        TicketStatus::Done,
        &trans,
    )
    .await?;

    // 修改书籍信息（库存减少）
    let book = entity::book::Entity::find_by_id(&order.book_isbn)
        .one(&trans)
        .await?
        .ok_or_else(|| not_found("Book not found"))?;

    let old_on_shelf_count = book.on_shelf_count;

    // 校验库存是否足够
    if old_on_shelf_count < order.total_count {
        return Err(unprocessable_entity("Not enough books on shelf").into());
    }

    let mut active_book = book.into_active_model();
    active_book.on_shelf_count = Set(old_on_shelf_count - order.total_count);
    active_book.update(&trans).await?;

    trans.commit().await?;

    Ok(AJson(order.into()))
}

#[p(
    responses(
        (status = OK, description = "Customer revoke order successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/sell/{id}/revoke")]
pub async fn revoke_sell(
    id: Path<i32>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    Ok(AJson(
        change_order_status(
            id.into_inner(),
            TicketStatus::Pending,
            TicketType::Sell,
            TicketStatus::Revoked,
            db.get_ref(),
        )
        .await?
        .into(),
    ))
}

#[p(
    request_body = NewOrder,
    responses(
        (status = OK, description = "Stock book successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/stock")]
pub async fn stock_book(
    order: AJson<NewOrder>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let order = order.into_inner();
    // 校验订单的合法性
    if order.total_count <= 0 {
        return Err(unprocessable_entity("Total count must be greater than 0").into());
    }

    let trans = db.begin().await?;
    // 获取或创建对应的书籍信息
    let book = entity::book::Entity::find_by_id(&order.book_isbn)
        .one(&trans)
        .await?;
    if book.is_none() {
        if let Some(book_info) = order.book.clone() {
            let mut active_book = book_info.into_active_model();
            active_book.isbn = Set(order.book_isbn.clone());
            active_book.inventory_count = Set(0);
            active_book.on_shelf_count = Set(0);
            active_book.insert(&trans).await?;
        } else {
            return Err(unprocessable_entity("Book not found and info not provided").into());
        }
    }

    // 创建订单
    let order = order
        .into_active_model(auth.auth_info.id, TicketType::Stock)
        .insert(&trans)
        .await?;

    // 提交更改
    trans.commit().await?;
    Ok(AJson(order.into()))
}

#[p(
    params(PagingRequest),
    responses(
        (status = OK, description = "Get stock list successfully", body = [GetOrder]),
    ),
    security(("jwt_token" = []))
)]
#[get("/stock")]
pub async fn get_stock_list(
    paging: Query<PagingRequest>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<HttpResponse> {
    entity::order_list::Entity::find()
        .filter(order_list::Column::Typ.eq(TicketType::Stock))
        .paged::<DatabaseConnection, _, GetOrder>(paging.into_inner(), db.get_ref())
        .await
}

#[p(
    responses(
        (status = OK, description = "We pay for the book successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/stock/{id}/pay")]
pub async fn pay_stock(
    id: Path<i32>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let trans = db.begin().await?;
    let order = pay_order(
        id.into_inner(),
        TicketStatus::Pending,
        TicketType::Stock,
        TicketStatus::StockPaid,
        &trans,
    )
    .await?;
    trans.commit().await?;
    Ok(AJson(order.into()))
}

#[p(
    responses(
        (status = OK, description = "We revoke the order successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/stock/{id}/revoke")]
pub async fn revoke_stock(
    id: Path<i32>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    Ok(AJson(
        change_order_status(
            id.into_inner(),
            TicketStatus::Pending,
            TicketType::Stock,
            TicketStatus::Revoked,
            db.get_ref(),
        )
        .await?
        .into(),
    ))
}

#[p(
    responses(
        (status = OK, description = "We confirm the order successfully", body = GetOrder),
    ),
    security(("jwt_token" = []))
)]
#[post("/stock/{id}/confirm")]
pub async fn confirm_stock(
    id: Path<i32>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let trans = db.begin().await?;
    // 修改状态
    let order = change_order_status(
        id.into_inner(),
        TicketStatus::StockPaid,
        TicketType::Stock,
        TicketStatus::Done,
        &trans,
    )
    .await?;
    // 修改库存
    let book = entity::book::Entity::find_by_id(&order.book_isbn)
        .one(&trans)
        .await?
        .ok_or_else(|| not_found("Cannot find the book"))?;
    let old_count = book.inventory_count;
    let mut book = book.into_active_model();
    book.inventory_count = Set(old_count + order.total_count);
    book.update(&trans).await?;

    trans.commit().await?;

    Ok(AJson(order.into()))
}

/// 根据指定条件修改订单记录。
///
/// 只会更新一次，不必要使用事务。
async fn change_order_status<C: ConnectionTrait>(
    id: i32,
    expected_status: TicketStatus,
    expected_type: TicketType,
    new_status: TicketStatus,
    db: &C,
) -> AResult<order_list::Model> {
    let order = order_list::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("Order not found"))?;
    if order.status == expected_status {
        if order.typ != expected_type {
            return Err(conflict(format!(
                "Order type is not {:?}, but {:?}",
                expected_type, order.typ
            ))
            .into());
        }
        let mut active_order = order.into_active_model();
        active_order.status = Set(new_status);
        active_order.updated_at = Set(Utc::now().naive_utc());
        Ok(active_order.update(db).await?)
    } else {
        Err(conflict(format!(
            "Order status is not {:?}, but {:?}",
            expected_status, order.status
        ))
        .into())
    }
}

/// 根据指定条件修改订单记录，并添加支付记录到 transaction 表。
///
/// 可能会更新多次，必须使用事务。
async fn pay_order(
    id: i32,
    expected_status: TicketStatus,
    expected_type: TicketType,
    new_status: TicketStatus,
    db: &sea_orm::DatabaseTransaction,
) -> AResult<order_list::Model> {
    // 修改状态
    let order = change_order_status(id, expected_status, expected_type, new_status, db).await?;
    // 添加支付记录
    let active_trans: entity::transaction::ActiveModel = order.clone().into();
    active_trans.insert(db).await?;

    Ok(order)
}

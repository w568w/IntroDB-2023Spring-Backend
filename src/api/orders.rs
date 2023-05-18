use crate::utils::errors::{bad_request, conflict, not_found, unprocessable_entity};
use crate::utils::jwt::{AllowAdmin, JwtClaims};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::web::Data;
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use entity::book::Model;
use entity::order_list::{GetOrder, NewOrder};
use entity::user::{GetUser, NewUser};
use entity::{book, order_list, TicketStatus};
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[p(
    responses(
        (status = OK, description = "Book sold successfully", body = GetOrder),
    ),
)]
#[post("/sell")]
pub async fn sell_book(book: AJson<NewOrder>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Get sell list successfully", body = [GetOrder]),
    ),
)]
#[get("/sell")]
pub async fn get_sell_list(paging: Query<PagingRequest>) -> AResult<AJson<Vec<GetOrder>>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Customer buy book successfully", body = GetOrder),
    ),
)]
#[post("/sell/{id}/pay")]
pub async fn pay_sell(id: Path<i32>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Customer revoke order successfully", body = GetOrder),
    ),
)]
#[post("/sell/{id}/revoke")]
pub async fn revoke_sell(id: Path<i32>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    request_body = NewOrder,
    responses(
        (status = OK, description = "Stock book successfully", body = GetOrder),
    ),
)]
#[post("/stock")]
pub async fn stock_book(
    order: AJson<NewOrder>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let order = order.into_inner();
    // 获取或创建对应的书籍信息
    let book = entity::book::Entity::find_by_id(&order.book_isbn)
        .one(db.get_ref())
        .await?;
    if book.is_none() {
        if let Some(book_info) = order.book.clone() {
            let mut active_book = book_info.into_active_model();
            active_book.isbn = Set(order.book_isbn.clone());
            active_book.inventory_count = Set(0);
            active_book.on_shelf_count = Set(0);
            active_book.insert(db.get_ref()).await?;
        } else {
            return Err(unprocessable_entity("Book not found and info not provided").into());
        }
    }

    // 创建订单
    Ok(AJson(
        order
            .into_active_model(auth.auth_info.id)
            .insert(db.get_ref())
            .await?
            .into(),
    ))
}

#[p(
    params(PagingRequest),
    responses(
        (status = OK, description = "Get stock list successfully", body = [GetOrder]),
    ),
)]
#[get("/stock")]
pub async fn get_stock_list(
    paging: Query<PagingRequest>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<Vec<GetOrder>>> {
    Ok(AJson(
        entity::order_list::Entity::find()
            .limit(paging.page_size)
            .offset(paging.page * paging.page_size)
            .all(db.get_ref())
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

#[p(
    responses(
        (status = OK, description = "We pay for the book successfully", body = GetOrder),
    ),
)]
#[post("/stock/{id}/pay")]
pub async fn pay_stock(
    id: Path<i32>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let trans = db.begin().await?;
    // 修改状态
    let order = change_order_status(
        id.into_inner(),
        TicketStatus::Pending,
        TicketStatus::StockPaid,
        &trans,
    )
    .await?;
    // 添加支付记录
    let active_trans: entity::transaction::ActiveModel = order.clone().into();
    active_trans.insert(&trans).await?;

    trans.commit().await?;

    Ok(AJson(order.into()))
}

#[p(
    responses(
        (status = OK, description = "We revoke the order successfully", body = GetOrder),
    ),
)]
#[post("/stock/{id}/revoke")]
pub async fn revoke_stock(
    id: Path<i32>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    Ok(AJson(
        change_order_status(
            id.into_inner(),
            TicketStatus::Pending,
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
)]
#[post("/stock/{id}/confirm")]
pub async fn confirm_stock(
    id: Path<i32>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetOrder>> {
    let trans = db.begin().await?;
    // 修改状态
    let order = change_order_status(
        id.into_inner(),
        TicketStatus::StockPaid,
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

async fn change_order_status<C: ConnectionTrait>(
    id: i32,
    expected_status: TicketStatus,
    new_status: TicketStatus,
    db: &C,
) -> AResult<order_list::Model> {
    let order = order_list::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| not_found("Order not found"))?;
    if order.status == expected_status {
        let mut active_order = order.into_active_model();
        active_order.status = Set(new_status);
        Ok(active_order.update(db).await?)
    } else {
        Err(conflict(format!(
            "Order status is not {:?}, but {:?}",
            expected_status, order.status
        ))
        .into())
    }
}

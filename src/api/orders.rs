use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use entity::book::Model;
use entity::order_list::{GetOrder, NewOrder};
use entity::user::{GetUser, NewUser};
use entity::{book, order_list};
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
pub async fn get_sell_list(
    paging: Query<PagingRequest>,
) -> AResult<AJson<Vec<GetOrder>>> {
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
    responses(
        (status = OK, description = "Stock book successfully", body = GetOrder),
    ),
)]
#[post("/stock")]
pub async fn stock_book(book: AJson<NewOrder>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Get stock list successfully", body = [GetOrder]),
    ),
)]
#[get("/stock")]
pub async fn get_stock_list(
    paging: Query<PagingRequest>,
) -> AResult<AJson<Vec<GetOrder>>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "We pay for the book successfully", body = GetOrder),
    ),
)]
#[post("/stock/{id}/pay")]
pub async fn pay_stock(id: Path<i32>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "We revoke the order successfully", body = GetOrder),
    ),
)]
#[post("/stock/{id}/revoke")]
pub async fn revoke_stock(id: Path<i32>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "We confirm the order successfully", body = GetOrder),
    ),
)]
#[post("/stock/{id}/confirm")]
pub async fn confirm_stock(id: Path<i32>) -> AResult<AJson<GetOrder>> {
    todo!()
}

#[derive(Deserialize, ToSchema)]
pub struct PutOnShelfRequest {
    put_count: i32,
}

#[p(
    request_body = PutOnShelfRequest,
    responses(
        (status = OK, description = "We put the book on the shelf successfully", body = Model),
    ),
)]
#[post("/stock/{id}/put_on_shelf")]
pub async fn put_on_shelf(
    id: Path<i32>,
    info: AJson<PutOnShelfRequest>,
) -> AResult<AJson<Model>> {
    todo!()
}

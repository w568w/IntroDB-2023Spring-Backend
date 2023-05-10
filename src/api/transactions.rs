use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use entity::transaction::GetTransaction;


#[p(
    responses(
        (status = OK, description = "Get transaction list successfully", body = [GetTransaction]),
    ),
)]
#[get("/transaction")]
pub async fn get_transaction_list(
    paging: Query<PagingRequest>,
) -> AResult<AJson<Vec<GetTransaction>>> {
    todo!()
}
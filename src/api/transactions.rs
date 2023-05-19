use super::preclude::*;

use super::{PagingRequest};
use actix_web::{
    get,
    web::{Query},
};
use entity::transaction::GetTransaction;


#[p(
    responses(
        (status = OK, description = "Get transaction list successfully", body = [GetTransaction]),
    ),
)]
#[get("/transaction")]
pub async fn get_transaction_list(
    _paging: Query<PagingRequest>,
) -> AResult<AJson<Vec<GetTransaction>>> {
    todo!()
}
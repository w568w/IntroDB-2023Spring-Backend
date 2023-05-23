use crate::utils::ext::SelectExt;
use crate::utils::jwt::{AllowAdmin, JwtClaims};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{get, web::Query};
use entity::transaction::GetTransaction;
use sea_orm::{DatabaseConnection, EntityTrait};

#[p(
    params(PagingRequest),
    responses(
        (status = OK, description = "Get transaction list successfully", body = [GetTransaction]),
    ),
)]
#[get("/transaction")]
pub async fn get_transaction_list(
    paging: Query<PagingRequest>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<HttpResponse> {
    entity::transaction::Entity::find()
        .paged::<DatabaseConnection, _, GetTransaction>(paging.into_inner(), db.get_ref())
        .await
}

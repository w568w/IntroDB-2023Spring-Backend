use crate::utils::jwt::{AllowAdmin, JwtClaims};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::{get, web::Query};
use entity::transaction::GetTransaction;
use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect};

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
) -> AResult<AJson<Vec<GetTransaction>>> {
    Ok(AJson(
        entity::transaction::Entity::find()
            .limit(paging.page_size)
            .offset(paging.page * paging.page_size)
            .all(db.get_ref())
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
    ))
}

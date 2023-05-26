use crate::utils::ext::SelectExt;
use crate::utils::jwt::{AllowAdmin, JwtClaims};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{get, web::Query};
use chrono::NaiveDateTime;
use entity::transaction::GetTransaction;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct TransactionFilter {
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
    #[serde(flatten)]
    pub paging: PagingRequest,
}

#[p(
    params(TransactionFilter),
    responses(
        (status = OK, description = "Get transaction list successfully", body = [GetTransaction]),
    ),
    security(("jwt_token" = []))
)]
#[get("/transaction")]
pub async fn get_transaction_list(
    params: Query<TransactionFilter>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<HttpResponse> {
    let params = params.into_inner();
    entity::transaction::Entity::find()
        .apply_if(params.from, |q, v| {
            q.filter(entity::transaction::Column::CreatedAt.gt(v))
        })
        .apply_if(params.to, |q, v| {
            q.filter(entity::transaction::Column::CreatedAt.lt(v))
        })
        .paged::<DatabaseConnection, _, GetTransaction>(params.paging, db.get_ref())
        .await
}

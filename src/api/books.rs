use crate::utils::errors::not_found;
use crate::utils::errors::unprocessable_entity;
use crate::utils::ext::OptionExt;
use crate::utils::ext::SelectExt;
use crate::utils::jwt::AllowAdmin;
use crate::utils::jwt::JwtClaims;
use crate::utils::permission::APermission;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{
    get, patch, post,
    web::{Path, Query},
};
use entity::book::{Model, UpdateBook};

use sea_orm::Set;
use sea_orm::Unchanged;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa::ToSchema;

#[derive(Deserialize, IntoParams)]
pub struct BookFilter {
    pub isbn: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    #[serde(flatten)]
    pub paging: PagingRequest,
}

#[p(
    params(BookFilter),
    responses(
        (status = OK, description = "Get books successful", body = [Model])
    ),
    security(("jwt_token" = []))
)]
#[get("/book")]
pub async fn get_books(
    data: Query<BookFilter>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<HttpResponse> {
    let mut query = entity::book::Entity::find();

    for (column, value) in &[
        (entity::book::Column::Isbn, &data.isbn),
        (entity::book::Column::Title, &data.title),
        (entity::book::Column::Author, &data.author),
        (entity::book::Column::Publisher, &data.publisher),
    ] {
        query = value
            .as_ref()
            .apply_if_some(query, |query, value| query.filter(column.contains(value)));
    }

    query
        .paged::<DatabaseConnection, _, Model>(data.into_inner().paging, db.get_ref())
        .await
}

#[p(
    request_body = UpdateBook,
    responses(
        (status = OK, description = "Update book successful", body = Model),
    ),
    security(("jwt_token" = []))
)]
#[patch("/book/{isbn}")]
pub async fn update_book(
    isbn: Path<String>,
    _auth: APermission<JwtClaims, AllowAdmin>,
    book: AJson<UpdateBook>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<Model>> {
    let mut active_book = book.into_inner().into_active_model();
    active_book.isbn = Unchanged(isbn.into_inner());

    Ok(AJson(active_book.update(db.get_ref()).await?))
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
#[post("/book/{isbn}/put_on_shelf")]
pub async fn put_on_shelf(
    isbn: Path<String>,
    info: AJson<PutOnShelfRequest>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<Model>> {
    let book = entity::book::Entity::find_by_id(isbn.into_inner())
        .one(db.get_ref())
        .await?
        .ok_or_else(|| not_found("Book not found"))?;
    if book.inventory_count < info.put_count {
        Err(unprocessable_entity("Inventory count is not enough").into())
    } else if book.on_shelf_count + info.put_count < 0 {
        Err(unprocessable_entity("On shelf count is not enough").into())
    } else {
        let old_inventory_count = book.inventory_count;
        let old_shelf_count = book.on_shelf_count;
        let mut active_book = book.into_active_model();
        active_book.inventory_count = Set(old_inventory_count - info.put_count);
        active_book.on_shelf_count = Set(old_shelf_count + info.put_count);
        Ok(AJson(active_book.update(db.get_ref()).await?))
    }
}

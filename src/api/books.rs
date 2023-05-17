use crate::utils::ext::OptionExt;

use super::preclude::*;

use super::PagingRequest;
use actix_web::web::Data;
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
};
use entity::book::{self, Model};
use entity::user::{GetUser, NewUser};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct BookFilter {
    pub isbn: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    #[serde(flatten)]
    pub paging: PagingRequest,
}

#[p(
    responses(
        (status = OK, description = "Get books successful", body = [Model])
    ),
)]
#[get("/book")]
pub async fn get_books(
    data: Query<BookFilter>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<Vec<Model>>> {
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
    query = query
        .limit(data.paging.page_size)
        .offset(data.paging.page_size * data.paging.page);

    Ok(AJson(query.all(db.as_ref()).await?))
}

#[p(
    responses(
        (status = OK, description = "Update book successful", body = Model),
    )
)]
#[patch("/book/{isbn}")]
pub async fn update_book(
    isbn: Path<String>,
    book: AJson<book::UpdateBook>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<Model>> {
    let mut active_book = book.into_inner().into_active_model();
    active_book.isbn = Set(isbn.into_inner());

    Ok(AJson(active_book.update(db.as_ref()).await?))
}

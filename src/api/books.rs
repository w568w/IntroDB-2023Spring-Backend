use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use entity::book::{self, Model};
use entity::user::{GetUser, NewUser};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[p(
    responses(
        (status = OK, description = "Get books successful", body = [Model])
    ),
)]
#[get("/book")]
pub async fn get_books(page: Query<PagingRequest> ) -> AResult<AJson<Vec<Model>>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Update book successful", body = Model),
    )
)]
#[patch("/book/{isbn}")]
pub async fn update_book(isbn: Path<String>, book: AJson<book::UpdateBook>) -> AResult<AJson<Model>> {
    todo!()
}
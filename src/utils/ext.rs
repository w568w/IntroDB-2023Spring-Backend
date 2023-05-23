use std::borrow::Borrow;

use actix_web::HttpResponse;
use async_trait::async_trait;
use sea_orm::{
    ConnectionTrait, EntityTrait, QuerySelect, QueryTrait, Select, StatementBuilder,
    TransactionTrait,
};
use serde::Serialize;

use crate::api::PagingRequest;

use super::errors::{internal_server_error, AResult};

pub trait OptionExt<T>
where
    Self: Sized,
{
    fn apply_if_some<Applied, F: FnOnce(Applied, T) -> Applied>(
        self,
        applied: Applied,
        f: F,
    ) -> Applied;
}

impl<T> OptionExt<T> for Option<T> {
    fn apply_if_some<Applied, F: FnOnce(Applied, T) -> Applied>(
        self,
        applied: Applied,
        f: F,
    ) -> Applied {
        match self {
            Some(option) => f(applied, option),
            None => applied,
        }
    }
}

#[async_trait]
pub trait SelectExt<T>
where
    T: EntityTrait,
{
    async fn paged<
        C: ConnectionTrait + TransactionTrait,
        D: Borrow<C> + Send + Sync,
        R: From<T::Model> + Serialize,
    >(
        self,
        request: PagingRequest,
        db: D,
    ) -> AResult<HttpResponse>;
}

#[async_trait]
impl<T> SelectExt<T> for Select<T>
where
    T: EntityTrait,
{
    async fn paged<
        C: ConnectionTrait + TransactionTrait,
        D: Borrow<C> + Send + Sync,
        R: From<T::Model> + Serialize,
    >(
        self,
        request: PagingRequest,
        db: D,
    ) -> AResult<HttpResponse> {
        let query = self.clone();
        let count_statement = self
            .select_only()
            .column_as(
                sea_orm::sea_query::Expr::count(sea_orm::sea_query::Expr::val(1)),
                "count",
            )
            .build(db.borrow().get_database_backend());
        let count_result = db
            .borrow()
            .query_one(count_statement)
            .await?
            .ok_or_else(|| internal_server_error("Unable to count"))?;

        let count: i32 = count_result.try_get("", "count")?;
        let query_result: Vec<R> = query
            .limit(request.page_size)
            .offset(request.page * request.page_size)
            .all(db.borrow())
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(HttpResponse::Ok()
            .append_header(("X-Item-Count", count))
            .json(query_result))
    }
}

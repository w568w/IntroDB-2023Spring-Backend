use std::{
    future::{ready, Future, Ready},
    pin::Pin,
    task::Poll,
};

use actix_web::{web::Data, FromRequest};
use either::Either;
use pin_project::pin_project;

use super::errors::{forbidden, internal_server_error};

pub struct AllRejected;

impl CheckPermission for AllRejected {
    type Output = ();
    type Future = std::future::Ready<Result<Option<Self::Output>, actix_web::Error>>;
    type Authentication = ();
    type AppData = ();
    fn check_permission(
        _: Data<Self::AppData>,
        _permission: &Self::Authentication,
    ) -> Self::Future {
        ready(Ok(None))
    }
}

pub trait CheckPermission {
    type Output;
    type Future: Future<Output = Result<Option<Self::Output>, actix_web::Error>>;
    type Authentication;
    type AppData: ?Sized + 'static;
    fn check_permission(
        req: Data<Self::AppData>,
        permission: &Self::Authentication,
    ) -> Self::Future;
}

pub struct APermission<Extractor, Checker>
where
    Extractor: FromRequest,
    Checker: CheckPermission<Authentication = Extractor>,
{
    pub extracted_info: Extractor,
    pub auth_info: Checker::Output,
}

/// A FromRequest trait for Data<T> that has special handling for empty type `Data<()>`.
trait EmptySafeDataFromRequest<T: ?Sized + 'static>: Sized {
    type Future: Future<Output = Result<Self, actix_web::Error>>;
    fn from_request_safe(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future;
}

impl<T: ?Sized + 'static> EmptySafeDataFromRequest<T> for Data<T> {
    type Future = Ready<Result<Self, actix_web::Error>>;
    default fn from_request_safe(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(<Data<T> as FromRequest>::from_request(req, payload).into_inner())
    }
}

impl EmptySafeDataFromRequest<()> for Data<()> {
    fn from_request_safe(
        _: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(Ok(Data::new(())))
    }
}

impl<E, T> FromRequest for APermission<E, T>
where
    E: FromRequest,
    T: CheckPermission<Authentication = E>,
{
    type Error = actix_web::error::Error;

    type Future =
        Either<std::future::Ready<Result<Self, Self::Error>>, PermissionExtractFunc<E, T>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let data = Data::<T::AppData>::from_request_safe(req, payload).into_inner();
        match data {
            Ok(data) => Either::Right(PermissionExtractFunc {
                data,
                fut: E::from_request(req, payload),
            }),
            Err(err) => Either::Left(ready(Err(internal_server_error(format!(
                "Failed to extract data from request: {:?}",
                err
            ))))),
        }
    }
}

#[pin_project]
pub struct PermissionExtractFunc<E, T>
where
    E: FromRequest,
    T: CheckPermission<Authentication = E>,
{
    #[pin]
    fut: E::Future,
    data: Data<T::AppData>,
}

impl<E, T> Future for PermissionExtractFunc<E, T>
where
    E: FromRequest,
    T: CheckPermission<Authentication = E>,
{
    type Output = Result<APermission<E, T>, <APermission<E, T> as FromRequest>::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let result = this.fut.poll(cx);
        match result {
            Poll::Ready(Ok(info)) => {
                let mut check_fn = Box::pin(T::check_permission(this.data.clone(), &info));
                let permitted = Pin::as_mut(&mut check_fn).poll(cx);
                match permitted {
                    Poll::Ready(Ok(Some(auth_info))) => Poll::Ready(Ok(APermission {
                        extracted_info: info,
                        auth_info,
                    })),
                    Poll::Ready(Ok(None)) => Poll::Ready(Err(forbidden("Permission denied"))),
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Pending => Poll::Pending,
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

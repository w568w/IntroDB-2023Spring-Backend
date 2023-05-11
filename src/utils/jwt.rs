use std::{future::Future, pin::Pin};

use actix_web::{web::Data, FromRequest};
use entity::user;
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use super::{permission::CheckPermission, errors::{unauthorized, internal_server_error, not_found}};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub exp: usize,
    pub iss: String,
    pub user_id: i32,
}

impl FromRequest for JwtClaims {
    type Error = actix_web::error::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;
    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "))
            .and_then(|token| {
                decode::<JwtClaims>(
                    token,
                    &DecodingKey::from_secret("secret".as_ref()),
                    &Validation::default(),
                )
                .ok()
            })
            .map(|data| data.claims);
        std::future::ready(match token {
            Some(token) => Ok(token),
            None => Err(unauthorized("Missing or bad token")),
        })
    }
}

trait JwtValidator {
    fn validate(model: &user::Model) -> bool;
}

impl<T> CheckPermission for T
where
    T: JwtValidator,
{
    type Authentication = JwtClaims;
    type Output = user::Model;
    type Future = Pin<Box<dyn Future<Output = Result<Option<user::Model>, actix_web::Error>>>>;
    type AppData = DatabaseConnection;
    fn check_permission(
        db: Data<DatabaseConnection>,
        permission: &Self::Authentication,
    ) -> Self::Future {
        let user_id = permission.user_id;
        Box::pin(async move {
            let this_user = user::Entity::find_by_id(user_id)
                .one(db.as_ref())
                .await
                .map_err(internal_server_error)?
                .ok_or_else(|| not_found("The user does not exist"))?;
            if Self::validate(&this_user) {
                Ok(Some(this_user))
            } else {
                Ok(None)
            }
        })
    }
}

pub struct AllowAdmin;
impl JwtValidator for AllowAdmin {
    fn validate(model: &user::Model) -> bool {
        model.role == "admin" || AllowSuperAdmin::validate(model)
    }
}

pub struct AllowSuperAdmin;
impl JwtValidator for AllowSuperAdmin {
    fn validate(model: &user::Model) -> bool {
        model.role == "super_admin"
    }
}
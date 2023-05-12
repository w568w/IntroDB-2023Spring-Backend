use std::{env, future::Future, pin::Pin};

use actix_web::{web::Data, FromRequest};
use chrono::Utc;
use entity::user;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    api::auth::JwtToken,
    contants::{
        envs::JWT_SECRET, user_type, ACCESS_TOKEN_EXPIRE_SECONDS, ISSUER,
        REFRESH_TOKEN_EXPIRE_SECONDS, SECRET_KEY_LENGTH,
    },
};

use super::{
    errors::{internal_server_error, not_found, unauthorized},
    permission::{self, CheckPermission},
};

fn sys_jwt_secret() -> String {
    env::var(JWT_SECRET).expect("Cannot get JWT secret key from environment variable")
}

pub fn gen_secret_key(different_from: Option<String>) -> String {
    loop {
        let key = OsRng
            .sample_iter(&Alphanumeric)
            .take(SECRET_KEY_LENGTH)
            .map(char::from)
            .collect();
        if different_from.is_none() || &key != different_from.as_ref().unwrap() {
            return key;
        }
    }
}

pub fn issue_acc_ref_token(
    user_id: i32,
    secret_key: String,
) -> Result<JwtToken, jsonwebtoken::errors::Error> {
    Ok(JwtToken {
        access_token: issue_token(
            user_id,
            secret_key.clone(),
            TokenType::Access,
            ACCESS_TOKEN_EXPIRE_SECONDS,
        )?,
        refresh_token: issue_token(
            user_id,
            secret_key,
            TokenType::Refresh,
            REFRESH_TOKEN_EXPIRE_SECONDS,
        )?,
    })
}

pub fn issue_token(
    user_id: i32,
    secret_key: String,
    token_type: TokenType,
    expire_period: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    JwtClaims {
        exp: Utc::now().timestamp() + expire_period,
        iss: ISSUER.to_owned(),
        user_id,
        secret_key,
        typ: token_type,
    }
    .try_into()
}

impl TryInto<String> for JwtClaims {
    type Error = jsonwebtoken::errors::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(sys_jwt_secret().as_bytes()),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub exp: i64,
    pub iss: String,
    pub user_id: i32,
    pub secret_key: String,
    pub typ: TokenType,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum TokenType {
    Access = 0,
    Refresh = 1,
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
                    &DecodingKey::from_secret(sys_jwt_secret().as_bytes()),
                    &Validation::default(),
                )
                .ok()
            })
            .map(|data| data.claims);

        if let Some(ref token) = token {
            // 验证 token 是否过期
            if token.exp < Utc::now().timestamp() {
                return std::future::ready(Err(unauthorized("Token expired")));
            }
        }
        std::future::ready(match token {
            Some(token) => Ok(token),
            None => Err(unauthorized("Missing or bad token")),
        })
    }
}

trait JwtValidator {
    fn validate(model: &user::Model, permission: &JwtClaims) -> bool;
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
        let permission = permission.clone();
        Box::pin(async move {
            let this_user = user::Entity::find_by_id(permission.user_id)
                .one(db.as_ref())
                .await
                .map_err(internal_server_error)?
                .ok_or_else(|| not_found("The user does not exist"))?;

            // 验证 Secret Key 是否匹配
            if this_user.secret_key != permission.secret_key {
                return Err(unauthorized("Invalid secret key"));
            }
            if Self::validate(&this_user, &permission) {
                Ok(Some(this_user))
            } else {
                Ok(None)
            }
        })
    }
}

pub struct AllowRefresh;
impl JwtValidator for AllowRefresh {
    fn validate(_: &user::Model, permission: &JwtClaims) -> bool {
        permission.typ == TokenType::Refresh
    }
}

pub struct AllowAdmin;
impl JwtValidator for AllowAdmin {
    fn validate(model: &user::Model, permission: &JwtClaims) -> bool {
        model.role == user_type::ADMIN || AllowSuperAdmin::validate(model, permission)
    }
}

pub struct AllowSuperAdmin;
impl JwtValidator for AllowSuperAdmin {
    fn validate(model: &user::Model, _: &JwtClaims) -> bool {
        model.role == user_type::SUPER_ADMIN
    }
}

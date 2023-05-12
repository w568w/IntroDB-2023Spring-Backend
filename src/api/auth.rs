use crate::contants::user_type;
use crate::utils::errors::{bad_request, conflict, forbidden, iam_a_teapot, not_found, AError};
use crate::utils::jwt::{
    gen_secret_key, issue_acc_ref_token, AllowAdmin, AllowRefresh, AllowSuperAdmin, JwtClaims,
};
use crate::utils::permission::APermission;

use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::web::{Data, Payload};
use actix_web::FromRequest;
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest,
};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use entity::user::{self, GetUser, NewUser, UpdateUser};
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

static ARGON: Lazy<Argon2> = Lazy::new(|| Argon2::default());

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    id: i32,
    password: String,
}

#[derive(Serialize, ToSchema)]
pub struct JwtToken {
    pub access_token: String,
    pub refresh_token: String,
}

fn to_salted_password(password: &String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(ARGON.hash_password(password.as_bytes(), &salt)?.to_string())
}

#[p(
    request_body = LoginRequest,
    responses(
        (status = OK, description = "Login successful", body = JwtToken),
        (status = IM_A_TEAPOT, description = "Invalid credentials"),
    ),
)]
#[post("/user/login")]
pub async fn login(
    creds: AJson<LoginRequest>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<JwtToken>> {
    // 验证用户存在及密码正确
    let user = user::Entity::find_by_id(creds.id)
        .one(db.as_ref())
        .await?
        .ok_or_else(|| not_found("User not found"))?;
    ARGON
        .verify_password(
            creds.password.as_bytes(),
            &PasswordHash::new(&user.password_salt)?,
        )
        .map_or_else(
            |err| match err {
                // 密码错误转换为响应
                argon2::password_hash::Error::Password => {
                    Err(AError::from(iam_a_teapot("Invalid credentials")))
                }
                // 其他错误视为服务器错误
                e => Err(e.into()),
            },
            |_| Ok(()),
        )?;

    // 分配 JWT
    Ok(AJson(issue_acc_ref_token(user.id, user.secret_key)?))
}

#[p(
    responses(
        (status = OK, description = "Refresh successful", body = JwtToken),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
)]
#[post("/user/refresh")]
pub async fn refresh(auth: APermission<JwtClaims, AllowRefresh>) -> AResult<AJson<JwtToken>> {
    Ok(AJson(issue_acc_ref_token(
        auth.auth_info.id,
        auth.auth_info.secret_key,
    )?))
}

#[p(
    responses(
        (status = OK, description = "Logout successful", body = GeneralResponse),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
)]
#[post("/user/logout")]
pub async fn logout(
    db: Data<DatabaseConnection>,
    auth: APermission<JwtClaims, AllowAdmin>,
) -> AResult<AJson<GeneralResponse>> {
    let mut user = auth.auth_info.into_active_model();
    user.secret_key = Set(gen_secret_key(user.secret_key.take()));
    user.update(db.as_ref()).await?;
    Ok(AJson(GeneralResponse {
        message: "Logout successful".to_string(),
    }))
}

#[p(
    request_body = NewUser,
    responses(
        (status = OK, description = "Register successful", body = GetUser),
        (status = BAD_REQUEST, description = "Invalid credentials", body = GeneralResponse),
        (status = CONFLICT, description = "User already exists", body = GeneralResponse),
    ),
)]
#[post("/user/register")]
pub async fn register(
    info: AJson<NewUser>,
    db: Data<DatabaseConnection>,
    req: HttpRequest,
    payload: Payload,
) -> AResult<AJson<GetUser>> {
    match info.role.as_str() {
        user_type::ADMIN => {
            // 只有超级管理员才能创建管理员
            let _ = APermission::<JwtClaims, AllowSuperAdmin>::from_request(
                &req,
                &mut payload.into_inner(),
            )
            .await?;
        }
        user_type::SUPER_ADMIN => {
            // 只有在无超级管理员的情况下才能创建超级管理员
            let su = user::Entity::find()
                .filter(user::Column::Role.eq(user_type::SUPER_ADMIN))
                .one(db.as_ref())
                .await?;
            if su.is_some() {
                return Err(conflict("Super admin already exists").into());
            }
        }
        _ => return Err(bad_request("Invalid user role").into()),
    }
    let mut info = info.into_inner();
    // 密码加盐
    info.password_salt = to_salted_password(&info.password_salt)?;
    // 初始化 Secret Key
    let mut active_info = info.into_active_model();
    active_info.secret_key = Set(gen_secret_key(None));
    // 储存用户
    let user = active_info.insert(db.as_ref()).await?;
    Ok(AJson(user.into()))
}

#[p(
    responses(
        (status = OK, description = "Get users successful", body = [GetUser])
    ),
)]
#[get("/user")]
pub async fn get_users(
    page: Query<PagingRequest>,
    _req: HttpRequest,
) -> AResult<AJson<Vec<GetUser>>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Get self successful", body = GetUser),
    ),
)]
#[get("/user/me")]
pub async fn get_self(auth: APermission<JwtClaims, AllowAdmin>) -> AResult<AJson<GetUser>> {
    Ok(AJson(auth.auth_info.into()))
}

#[p(
    responses(
        (status = OK, description = "Get user successful", body = GetUser),
        (status = NOT_FOUND, description = "User not found", body = GeneralResponse),
    ),
)]
#[get("/user/{id}")]
pub async fn get_user(
    id: Path<i32>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetUser>> {
    let id = id.into_inner();
    // 只有自己或超级管理员才能获取用户信息
    if auth.auth_info.role != user_type::SUPER_ADMIN && auth.auth_info.id != id {
        Err(forbidden("Permission denied").into())
    } else {
        let user = user::Entity::find_by_id(id)
            .one(db.as_ref())
            .await?
            .ok_or_else(|| not_found("User not found"))?;
        Ok(AJson(user.into()))
    }
}

#[p(
    responses(
        (status = OK, description = "Update user successful", body = GetUser),
        (status = BAD_REQUEST, description = "Invalid credentials", body = GeneralResponse),
        (status = NOT_FOUND, description = "User not found", body = GeneralResponse),
    ),
)]
#[patch("/user/{id}")]
pub async fn update_user(
    id: Path<i32>,
    mut info: AJson<UpdateUser>,
    auth: APermission<JwtClaims, AllowAdmin>,
    db: Data<DatabaseConnection>,
) -> AResult<AJson<GetUser>> {
    let id = id.into_inner();
    // 只有自己或超级管理员才能修改用户信息
    if auth.auth_info.role != user_type::SUPER_ADMIN && auth.auth_info.id != id {
        Err(forbidden("Permission denied").into())
    } else {
        if let Some(ref password) = info.password_salt {
            // 如果有密码，需要给密码加盐
            info.password_salt = Some(to_salted_password(password)?);
        }
        let mut info = info.into_inner().into_active_model();
        info.id = Set(id);
        let result = info.update(db.as_ref()).await;
        match result {
            Ok(user) => Ok(AJson(user.into())),
            Err(DbErr::RecordNotFound(_)) => Err(not_found("User not found").into()),
            Err(e) => Err(e.into()),
        }
    }
}

#[p(
    responses(
        (status = OK, description = "Delete user successful", body = GeneralResponse),
        (status = NOT_FOUND, description = "User not found", body = GeneralResponse),
    ),
)]
#[delete("/user/{id}")]
pub async fn delete_user(id: Path<i32>, _req: HttpRequest) -> AResult<AJson<GeneralResponse>> {
    todo!()
}

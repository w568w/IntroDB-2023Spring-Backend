use super::preclude::*;

use super::{GeneralResponse, PagingRequest};
use actix_web::{
    delete, get, patch, post,
    web::{Path, Query},
    HttpRequest, HttpResponse, Responder,
};
use entity::user::{GetUser, NewUser};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    id: String,
    password: String,
}

#[derive(Serialize, ToSchema)]
pub struct JwtToken {
    access_token: String,
    refresh_token: String,
}

#[p(
    request_body = LoginRequest,
    responses(
        (status = OK, description = "Login successful", body = JwtToken),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
)]
#[post("/user/login")]
pub async fn login(creds: AJson<LoginRequest>) -> AResult<AJson<JwtToken>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Refresh successful", body = JwtToken),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
)]
#[post("/user/refresh")]
pub async fn refresh(_req: HttpRequest) -> AResult<AJson<JwtToken>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Logout successful", body = GeneralResponse),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
)]
#[post("/user/logout")]
pub async fn logout(_req: HttpRequest) -> AResult<AJson<GeneralResponse>> {
    todo!()
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
pub async fn register(info: AJson<NewUser>, _req: HttpRequest) -> AResult<AJson<GetUser>> {
    todo!()
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
pub async fn get_self(_req: HttpRequest) -> AResult<AJson<GetUser>> {
    todo!()
}

#[p(
    responses(
        (status = OK, description = "Get user successful", body = GetUser),
        (status = NOT_FOUND, description = "User not found", body = GeneralResponse),
    ),
)]
#[get("/user/{id}")]
pub async fn get_user(id: Path<i32>, _req: HttpRequest) -> AResult<AJson<GetUser>> {
    todo!()
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
    info: AJson<NewUser>,
    _req: HttpRequest,
) -> AResult<AJson<GetUser>> {
    todo!()
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

use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};

pub mod auth;
pub mod books;
pub mod orders;
mod preclude;
pub mod stats;
pub mod transactions;

#[derive(Serialize, ToSchema)]
pub struct GeneralResponse {
    pub message: String,
}

#[serde_as]
#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PagingRequest {
    // 从字符串解析数据类型，是Query中无法使用#[serde(flatten)]的临时解决方案
    // 见 https://docs.rs/serde_qs/latest/serde_qs/index.html#flatten-workaround。
    #[serde_as(as = "DisplayFromStr")]
    pub page: u64,
    #[serde_as(as = "DisplayFromStr")]
    pub page_size: u64,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::login,
        auth::refresh,
        auth::logout,
        auth::register,
        auth::get_users,
        auth::get_self,
        auth::get_user,
        auth::update_user,
        auth::delete_user,
        books::get_books,
        books::update_book,
        books::put_on_shelf,
        orders::sell_book,
        orders::get_sell_list,
        orders::pay_sell,
        orders::revoke_sell,
        orders::stock_book,
        orders::get_stock_list,
        orders::pay_stock,
        orders::revoke_stock,
        orders::confirm_stock,
        transactions::get_transaction_list,
        stats::stat_transaction,
        stats::stat_stock,
        stats::stat_sell,
        stats::stat_book,
        stats::stat_bestsell,
    ),
    components(schemas(
        auth::LoginRequest,
        auth::JwtToken,
        books::PutOnShelfRequest,
        stats::StatSpan,
        stats::StatTransaction,
        stats::StatStock,
        stats::StatSell,
        stats::StatBook,
        stats::StatBestsell,
        GeneralResponse,
        PagingRequest,
        entity::user::GetUser,
        entity::user::NewUser,
        entity::user::UpdateUser,
        entity::book::Model,
        entity::book::UpdateBook,
        entity::book::NewBookInfo,
        entity::order_list::GetOrder,
        entity::order_list::NewOrder,
        entity::transaction::GetTransaction,
        entity::TicketStatus,
        entity::TicketType,
        entity::Sex,
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since it is already a registered component.
        components.add_security_scheme(
            "jwt_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        )
    }
}

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(auth::login)
            .service(auth::refresh)
            .service(auth::logout)
            .service(auth::register)
            .service(auth::get_users)
            .service(auth::get_self)
            .service(auth::get_user)
            .service(auth::update_user)
            .service(auth::delete_user)
            .service(books::get_books)
            .service(books::update_book)
            .service(books::put_on_shelf)
            .service(orders::sell_book)
            .service(orders::get_sell_list)
            .service(orders::pay_sell)
            .service(orders::revoke_sell)
            .service(orders::stock_book)
            .service(orders::get_stock_list)
            .service(orders::pay_stock)
            .service(orders::revoke_stock)
            .service(orders::confirm_stock)
            .service(transactions::get_transaction_list)
            .service(stats::stat_transaction)
            .service(stats::stat_stock)
            .service(stats::stat_sell)
            .service(stats::stat_book)
            .service(stats::stat_bestsell);
    }
}

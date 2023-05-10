use actix_web::web::ServiceConfig;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

pub mod auth;
pub mod books;
pub mod orders;
mod preclude;
pub mod transactions;

#[derive(Serialize, ToSchema)]
pub struct GeneralResponse {
    message: String,
}

#[derive(Deserialize, ToSchema)]
pub struct PagingRequest {
    page: u32,
    page_size: u32,
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
        auth::update_user,
        auth::delete_user,
        books::get_books,
        books::update_book,
        orders::sell_book,
        orders::get_sell_list,
        orders::pay_sell,
        orders::revoke_sell,
        orders::stock_book,
        orders::get_stock_list,
        orders::pay_stock,
        orders::revoke_stock,
        orders::confirm_stock,
        orders::put_on_shelf,
        transactions::get_transaction_list,
    ),
    components(schemas(
        auth::LoginRequest,
        auth::JwtToken,
        orders::PutOnShelfRequest,
        GeneralResponse,
        PagingRequest,
        entity::user::GetUser,
        entity::user::NewUser,
        entity::book::Model,
        entity::book::UpdateBook,
        entity::order_list::GetOrder,
        entity::order_list::NewOrder,
        entity::transaction::GetTransaction,
        entity::TicketStatus,
        entity::Sex,
    ))
)]
pub struct ApiDoc;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(auth::login)
            .service(auth::refresh)
            .service(auth::logout)
            .service(auth::register)
            .service(auth::get_users)
            .service(auth::get_self)
            .service(auth::update_user)
            .service(auth::delete_user)
            .service(books::get_books)
            .service(books::update_book)
            .service(orders::sell_book)
            .service(orders::get_sell_list)
            .service(orders::pay_sell)
            .service(orders::revoke_sell)
            .service(orders::stock_book)
            .service(orders::get_stock_list)
            .service(orders::pay_stock)
            .service(orders::revoke_stock)
            .service(orders::confirm_stock)
            .service(orders::put_on_shelf)
            .service(transactions::get_transaction_list);
    }
}

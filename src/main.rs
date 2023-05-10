use actix_web::{App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().configure(api::configure()).service(
            SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", api::ApiDoc::openapi()),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

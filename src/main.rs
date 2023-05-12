#![feature(min_specialization, ready_into_inner)]
use std::env;

use actix_web::{web, App, HttpServer};
use log::{error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
mod api;
mod utils;
mod contants;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Some(err) = dotenv::dotenv().err() {
        eprintln!("Error loading .env file: {}. You may safely ignore this error if you are not using .env file.", err);
    }
    let _ = env_logger::try_init();

    let db = Database::connect(
        env::var("DB_URL").expect("Unable to obtain DB_URL from the environment variables"),
    )
    .await
    .expect("Unable to connect to the database");

    info!("Starting migration");
    let result = Migrator::up(&db, None).await;
    if let Err(err) = result {
        error!("Error migrating, exiting: {}", err);
        return Ok(());
    } else {
        info!("Migration completed");
    }

    HttpServer::new(move || {
        App::new()
            .configure(api::configure())
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/api-docs/openapi.json", api::ApiDoc::openapi()),
            )
            .app_data(web::Data::new(db.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

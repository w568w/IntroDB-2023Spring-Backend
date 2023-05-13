#![feature(min_specialization, ready_into_inner)]
use std::env;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::{error, info, warn};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::contants::envs;
mod api;
mod contants;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Some(err) = dotenv::dotenv().err() {
        eprintln!("Error loading .env file: {}. You may safely ignore this error if you are not using .env file.", err);
    }
    let _ = env_logger::try_init();

    let db = Database::connect(
        env::var(envs::DB_URL).expect("Unable to obtain DB_URL from the environment variables"),
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

    let allow_cors: bool = env::var(envs::ALLOW_ALL_CORS)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);

    if allow_cors {
        warn!("CORS is enabled for all origins, this is not recommended for production!")
    }

    HttpServer::new(move || {
        let mut cors = Cors::default();
        if allow_cors {
            cors = cors
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(None);
        }
        App::new()
            .wrap(cors)
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

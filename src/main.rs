#![feature(min_specialization, ready_into_inner)]
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use log::{error, info, warn};
use migration::{Migrator, MigratorTrait};
use mimalloc::MiMalloc;
use sea_orm::Database;
use std::env;
use tokio::sync::Mutex;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::contants::envs;
mod api;
mod contants;
mod utils;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

    let redis_url = env::var(envs::REDIS_URL);
    let mut redis_conn = None;
    if let Ok(redis_url) = redis_url {
        let redis = redis::Client::open(redis_url).expect("Unable to open redis client");
        redis_conn = Some(
            redis
                .get_multiplexed_async_connection()
                .await
                .expect("Unable to connect to redis"),
        );
    }

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
        cors = cors.expose_headers([crate::contants::ITEM_COUNT_HEADER]);
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
            .app_data(web::Data::new(redis_conn.clone().map(Mutex::new)))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::future_not_send)]

use actix_web::web;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod routes;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = PgPoolOptions::new().connect(&conn_str).await?;

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Compress::default())
            .data(pool.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(
                web::get().to(|| {
                    actix_web::HttpResponse::NotFound().body("this resource does not exist.")
                }),
            ))
    })
    .bind(format!("0.0.0.0:{}", env::var("KYUSA_PORT")?))?
    .run()
    .await?;
    Ok(())
}

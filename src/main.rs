#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::integer_arithmetic,
    clippy::implicit_return,
    clippy::float_arithmetic,
    clippy::panic,
    clippy::expect_used,
    clippy::future_not_send,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::as_conversions,
    dead_code
)]

use actix_web::web;
use dotenv::dotenv;
use std::env;

mod routes;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

async fn p404() -> &'static str { "this resource does not exist." }

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let conn_str =
        std::env::var("DATABASE_URL").expect("Env var DATABASE_URL is required for this example.");
    let pool = sqlx::PgPool::new(&conn_str).await?;

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(pool.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind(format!("0.0.0.0:{}", env::var("KYUSA_PORT")?))?
    .run()
    .await?;
    Ok(())
}

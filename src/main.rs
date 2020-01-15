use actix_web::web;
use sqlx::PgPool;
use utils::config::Config;

mod routes;
mod utils;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

async fn p404() -> &'static str { "this resource does not exist." }

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::new("postgres://postgres:8j4y7xAA@localhost/kyous").await?;

    let config = Config::load().await.unwrap();
    let port = config.port.clone();

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    if !std::path::Path::new("./tmp").exists() {
        std::fs::create_dir_all("./tmp")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(config.clone())
            .data(pool.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await?;
    Ok(())
}

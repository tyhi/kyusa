#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::perf)]
#![allow(clippy::future_not_send)]

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use actix_web::web;

mod routes;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[actix_web::main]
async fn main() -> Result<()> {
    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Compress::default())
            .service(routes::routes())
            .default_service(web::resource("").route(
                web::get().to(|| {
                    actix_web::HttpResponse::NotFound().body("this resource does not exist.")
                }),
            ))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await?;
    Ok(())
}

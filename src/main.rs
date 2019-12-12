use actix_web::web;
use once_cell::sync::Lazy;
use sled::Db;
use utils::config::Config;

mod routes;
mod utils;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub static GLOBAL_DB: Lazy<Db> = Lazy::new(|| Db::open("db").unwrap());

async fn p404() -> &'static str { "this resource does not exist." }

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = Config::load().await.unwrap();

    let port = config.port.clone();
    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    if !std::path::Path::new("./tmp").exists() {
        std::fs::create_dir_all("./tmp")?;
    }

    // This is just warming up the database before it could get used.
    if let Err(why) = GLOBAL_DB.flush() {
        println!("{:?}", why);
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(config.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .start()
    .await
}

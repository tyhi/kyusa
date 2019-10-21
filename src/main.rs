use actix_web::{web, FromRequest};
use sled::Db;

mod cf_file_purge;
mod cfg;
mod dbu;
mod routes;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn p404() -> &'static str {
    "this resource does not exist."
}

fn main() -> std::io::Result<()> {
    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    if !std::path::Path::new("./tmp").exists() {
        std::fs::create_dir_all("./tmp")?;
    }

    let db = Db::open("db").unwrap();

    let config = cfg::load_cfg(db.clone()).unwrap();

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(db.clone())
            .data(config.clone())
            // Not using a defined temp folder caused issues on my arch linux server but not any others.
            .data(awmp::Parts::configure(|cfg| cfg.with_temp_dir("./tmp")))
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind("0.0.0.0:3000")?
    .run()
}

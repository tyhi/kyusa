use actix_web::{web, FromRequest};
use routes::{delete, serve, stats, upload};
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

fn main() {
    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads").unwrap();
    }

    let db = Db::open("db").unwrap();

    let config = cfg::load_cfg(db.clone());

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(db.clone())
            .data(config.clone())
            // Not using a defined temp folder caused issues on my arch linux server but not any others.
            .data(awmp::Parts::configure(|cfg| tempfile_loc(cfg)))
            .route("/u", actix_web::web::post().to(upload::upload))
            .route("/d/{folder}/{file}", web::get().to(delete::delete))
            .route("/stats", web::get().to(stats::stats))
            .service(web::resource("/{folder}/{file}").route(web::get().to(serve::serve)))
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .unwrap();
}

#[cfg(unix)]
fn tempfile_loc(cfg: awmp::PartsConfig) -> awmp::PartsConfig {
    cfg.with_temp_dir("/dev/shm")
}

#[cfg(target_os = "windows")]
fn tempfile_loc(cfg: awmp::PartsConfig) -> awmp::PartsConfig {
    cfg
}

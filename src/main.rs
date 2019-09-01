use actix_web::{web, FromRequest};
use routes::{delete, serve, stats, upload};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs;

mod dbu;
mod routes;

#[derive(Serialize, Clone, Deserialize)]
struct ServerSettings {
    api_keys: Vec<String>,
    admin_keys: Vec<String>,
    website_name: String,
    https: bool,
}

fn p404() -> &'static str {
    "this resource does not exist."
}

fn main() {
    if !std::path::Path::new("./config.json").exists() {
        panic!("no config");
    }
    let config_json = fs::File::open("./config.json").unwrap();
    let server_settings: ServerSettings = serde_json::from_reader(config_json).unwrap();

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads").unwrap();
    }

    let db = Db::open("db").unwrap();
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(db.clone())
            .data(server_settings.clone())
            .data(awmp::Parts::configure(|cfg| cfg.with_temp_dir("./uploads")))
            .route("/u", actix_web::web::post().to(upload::upload))
            .route("/d/{folder}/{file}", web::get().to(delete::delete))
            .route("/stats", web::get().to(stats::stats))
            // .service(web::resource("/lg/{folder}/{file}").route(web::get().to(looking_glass)))
            .service(web::resource("/{folder}/{file}").route(web::get().to(serve::serve)))
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .unwrap();
}

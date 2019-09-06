use actix_web::{web, FromRequest};
use routes::{delete, serve, stats, upload};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs;

mod cf_file_purge;
mod dbu;
mod routes;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub api_keys: Vec<String>,
    pub admin_keys: Vec<String>,
    pub website_name: String,
    pub https: bool,
    pub cloudflare: CloudFlareSettings,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CloudFlareSettings {
    pub domain: String,
    pub enabled: bool,
    pub cf_key: String,
    pub zone: Option<String>,
}

fn p404() -> &'static str {
    "this resource does not exist."
}

fn main() {
    if !std::path::Path::new("./config.json").exists() {
        log::error!("no config");
    }
    let config_json = fs::File::open("./config.json").unwrap();
    let mut server_settings: ServerSettings = serde_json::from_reader(config_json).unwrap();

    server_settings.cloudflare.zone = match server_settings.cloudflare.zone {
        Some(e) => Some(e),
        None => Some(
            cf_file_purge::get_domain_id(
                &server_settings.cloudflare.domain,
                &server_settings.cloudflare.cf_key,
            )
            .expect("unable to find cloudflare zone double check api key and website name"),
        ),
    };

    fs::write(
        "./config.json",
        serde_json::to_string_pretty(&server_settings.clone())
            .unwrap()
            .as_bytes(),
    )
    .unwrap();

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

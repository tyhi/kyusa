use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Stats {
    files: usize,
    version: String,
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn stats(database: web::Data<sled::Db>) -> HttpResponse {
    HttpResponse::Ok().json(Stats {
        files: database.len(),
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
    })
}

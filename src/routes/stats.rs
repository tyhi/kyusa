use crate::built_info;
use actix_web::{web, HttpResponse};
use serde::{Serialize};

#[derive(Serialize)]
struct Stats {
    files: usize,
    version: String,
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

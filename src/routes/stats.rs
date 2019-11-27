use crate::built_info;
use actix_web::{get, web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct Stats {
    files: usize,
    version: String,
    rustc: String,
}

#[get("/stats")]
pub async fn stats(database: web::Data<sled::Db>) -> HttpResponse {
    HttpResponse::Ok().json(Stats {
        files: database.len() - 1,
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
        rustc: built_info::RUSTC_VERSION.to_owned(),
    })
}

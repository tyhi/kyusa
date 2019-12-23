use crate::{built_info, GLOBAL_DB};
use actix_web::{get, HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
struct Stats {
    files: usize,
    version: String,
    rustc: String,
}

#[get("/stats")]
pub async fn stats() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(Stats {
        files: GLOBAL_DB.len(),
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
        rustc: built_info::RUSTC_VERSION.to_owned(),
    }))
}

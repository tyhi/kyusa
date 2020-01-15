use crate::{built_info, utils::database};
use actix_web::{error, get, web::Data, HttpResponse, Result};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
struct Stats {
    files: i64,
    version: String,
    rustc: String,
}

#[get("/stats")]
pub async fn stats(p: Data<PgPool>) -> Result<HttpResponse> {
    let files = match database::file_count(p).await {
        Ok(x) => x,
        Err(e) => return Err(error::ErrorInternalServerError(e)),
    };

    Ok(HttpResponse::Ok().json(Stats {
        files,
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
        rustc: built_info::RUSTC_VERSION.to_owned(),
    }))
}

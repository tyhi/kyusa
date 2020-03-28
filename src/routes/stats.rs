use crate::{built_info, utils::db};
use actix_web::{error, get, web::Data, HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
struct Stats {
    files: i64,
    users: i64,
    storesize: String,
    served: String,
    version: String,
    rustc: String,
}

#[get("/stats")]
pub async fn stats(db: Data<sled::Db>) -> Result<HttpResponse> {
    let metrics = db::get_metrics(db)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(Stats {
        files: metrics.files,
        users: metrics.users,
        storesize: format!("{:.2} MB", metrics.stored),
        served: format!("{:.2} MB", metrics.served),
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
        rustc: built_info::RUSTC_VERSION.to_owned(),
    }))
}

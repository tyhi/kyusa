use crate::utils::database;
use actix_files::NamedFile;
use actix_web::{error, get, web, Result};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}

#[get("/u/{folder}/{file}")]
pub async fn serve(info: web::Path<FilePath>, p: web::Data<PgPool>) -> Result<NamedFile> {
    if let Err(why) = database::inc_file(p, format!("/{}/{}", info.folder, info.file)).await {
        return Err(error::ErrorInternalServerError(why));
    };

    Ok(NamedFile::open(format!(
        "./uploads/{}/{}",
        info.folder, info.file
    ))?)
}

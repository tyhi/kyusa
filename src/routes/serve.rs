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

#[get("/{folder}/{file}")]
pub async fn serve(info: web::Path<FilePath>, p: web::Data<PgPool>) -> Result<NamedFile> {
    database::inc_file(p, format!("/{}/{}", info.folder, info.file))
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(NamedFile::open(format!(
        "./uploads/{}/{}",
        info.folder, info.file
    ))?)
}

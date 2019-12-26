use actix_files::NamedFile;
use actix_web::{get, web, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}

#[get("/u/{folder}/{file}")]
pub async fn serve(info: web::Path<FilePath>) -> Result<NamedFile> {
    Ok(NamedFile::open(format!(
        "./uploads/{}/{}",
        info.folder, info.file
    ))?)
}

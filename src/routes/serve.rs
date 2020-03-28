use crate::utils::db;
use actix_files::NamedFile;
use actix_web::{error, get, web, web::Data, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}

#[get("/{folder}/{file}")]
pub async fn serve(info: web::Path<FilePath>, db: Data<sled::Db>) -> Result<NamedFile> {
    if !std::path::Path::new(&format!("./uploads/{}/{}", info.folder, info.file)).exists() {
        return Err(error::ErrorNotFound("file does not exist"));
    }

    db::inc_file(db, format!("/{}/{}", info.folder, info.file))
        .await
        .map_err(|_| error::ErrorNotFound("file does not exist"))?;

    Ok(NamedFile::open(format!(
        "./uploads/{}/{}",
        info.folder, info.file
    ))?)
}

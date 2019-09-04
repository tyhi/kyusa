use actix_files::NamedFile;
use actix_web::{error, web, Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}

pub fn serve(info: web::Path<FilePath>) -> Result<NamedFile, Error> {
    let file = format!("./uploads/{}/{}", info.folder, info.file);

    match NamedFile::open(file) {
        Ok(e) => Ok(e),
        Err(_e) => Err(error::ErrorNotFound(
            "the file you are looking for is either deleted or never existed in the first place",
        )),
    }
}

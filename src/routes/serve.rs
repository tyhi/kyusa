use actix_files::NamedFile;
use actix_web::web;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}

pub fn serve(info: web::Path<FilePath>) -> actix_web::Result<NamedFile, actix_web::HttpResponse> {
    let file = format!("./uploads/{}/{}", info.folder, info.file);

    match NamedFile::open(file) {
        Ok(e) => Ok(e),
        Err(_e) => Err(actix_web::HttpResponse::NotFound().body(
            "the file you are looking for is either deleted or never existed in the first place",
        )),
    }
}

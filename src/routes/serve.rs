use crate::utils::{db, ENCODER};
use actix_files::NamedFile;
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web::{Data, Path},
    Result,
};
use sled::Db;

#[get("/{file}")]
pub async fn serve(info: Path<String>, db: Data<Db>) -> Result<NamedFile> {
    let id = if info.contains('.') {
        info.split('.')
            .next()
            .ok_or_else(|| ErrorNotFound("invalid url"))?
    } else {
        info.as_str()
    };

    // Get file from database
    let file = db::get(
        ENCODER
            .decode_url(id.into())
            .map_err(|_| ErrorNotFound("invalid url"))? as i64,
        db.open_tree("files")
            .map_err(actix_web::error::ErrorInternalServerError)?,
    )
    .await
    .ok_or_else(|| ErrorNotFound("file does not exist"))?;

    if file.deleted {
        return Err(ErrorNotFound(
            "the file you are trying to access has been deleted from the server.",
        ));
    }

    Ok(NamedFile::open(["./uploads/", &file.hash].concat())?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename([id, ".", &file.ext].concat())],
        })
        .set_content_type(file.mime.parse().map_err(ErrorInternalServerError)?))
}

#[get("/t/{file}")]
pub async fn serve_tmp(info: Path<String>, db: Data<Db>) -> Result<NamedFile> {
    // Get file from database
    let file = db::get_tmp(
        &info,
        db.open_tree("tmp")
            .map_err(actix_web::error::ErrorInternalServerError)?,
    )
    .await
    .ok_or_else(|| ErrorNotFound("file does not exist"))?;

    if file.deleted {
        return Err(ErrorNotFound(
            "the file you are trying to access has been deleted from the server.",
        ));
    }

    Ok(NamedFile::open(["./uploads/", &file.hash].concat())?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename(info.to_string())],
        })
        .set_content_type(file.mime.parse().map_err(ErrorInternalServerError)?))
}

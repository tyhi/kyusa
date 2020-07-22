use crate::utils::{db, ENCODER};
use actix_files::NamedFile;
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web::{Data, Path},
    Result,
};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct FilePath {
    pub file: String,
}

#[get("/{file}")]
pub async fn serve(info: Path<FilePath>, db: Data<PgPool>) -> Result<NamedFile> {
    let id = info
        .file
        .split('.')
        .next()
        .ok_or_else(|| ErrorNotFound("invalid url"))?;

    // Get file from database
    let file = db::get(
        ENCODER
            .decode_url(id.into())
            .map_err(|_| ErrorNotFound("invalid url"))? as i64,
        db,
    )
    .await
    .map_err(ErrorInternalServerError)?
    .ok_or_else(|| ErrorNotFound("file does not exist"))?;

    if file.deleted {
        return Err(ErrorNotFound(
            "the file you are trying to access has been deleted from the server.",
        ));
    }

    Ok(NamedFile::open(format!("./uploads/{}", file.hash))?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Inline,
            parameters: vec![DispositionParam::Filename(format!("{}.{}", id, file.ext))],
        })
        .set_content_type(file.mime.parse().map_err(ErrorInternalServerError)?))
}

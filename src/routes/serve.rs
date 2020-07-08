use crate::utils::{db, ENCODER};
use actix_files::NamedFile;
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web,
    web::Data,
    Result,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::{ffi::OsStr, path::Path};

#[derive(Deserialize)]
pub struct FilePath {
    pub file: String,
}

#[get("/{file}")]
pub async fn serve(info: web::Path<FilePath>, db: Data<PgPool>) -> Result<NamedFile> {
    let path = Path::new(&info.file)
        .file_stem()
        .and_then(OsStr::to_str)
        .map_or_else(|| "".to_string(), std::string::ToString::to_string);

    // Get file from database
    let file = db::get(ENCODER.decode_url(path.clone()).unwrap() as i64, db)
        .await
        .map_err(ErrorInternalServerError)?
        .ok_or_else(|| ErrorNotFound("file does not exist"))?;

    if file.deleted {
        return Err(ErrorNotFound(
            "the file you are trying to access has been deleted from the server.",
        ));
    }

    let dis = ContentDisposition {
        disposition: DispositionType::Inline,
        parameters: vec![DispositionParam::Filename(format!("{}.{}", path, file.ext))],
    };

    let e = NamedFile::open(format!("./uploads/{}", file.hash))?
        .set_content_disposition(dis)
        .set_content_type(file.mime.parse().map_err(ErrorInternalServerError)?);
    Ok(e)
}

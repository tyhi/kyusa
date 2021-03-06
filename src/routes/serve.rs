use crate::utils::{db::SLED, ENCODER};
use actix_files::NamedFile;
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    web::Path,
    Result,
};

#[get("/{file}")]
pub async fn serve(info: Path<String>) -> Result<NamedFile> {
    let id = if info.contains('.') {
        info.split('.')
            .next()
            .ok_or_else(|| ErrorNotFound("invalid url"))?
    } else {
        info.as_str()
    };

    // Get file from database
    let file = SLED
        .get(
            ENCODER
                .decode_url(id.into())
                .map_err(|_| ErrorNotFound("invalid url"))? as u64,
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

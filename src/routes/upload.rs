use crate::utils::ENCODER;
use actix_multipart::{Field, Multipart};
use actix_web::{error, post, web::Data, HttpRequest, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use sqlx::PgPool;
use std::{ffi::OsStr, path::Path};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Serialize)]
struct UploadResp {
    url: String,
}

struct NamedReturn {
    new_path: String,
    temp_path: String,
    uri: String,
    ext: String,
}

const RANDOM_FILE_EXT: &[&str] = &["png", "jpeg", "jpg", "webm", "gif", "avi", "mp4"];

#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
#[post("")]
pub async fn upload(
    mut multipart: Multipart,
    db: Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse> {
    // Handle multipart upload(s) field
    if let Ok(Some(file)) = multipart.try_next().await {
        let mut file: Field = file;
        let content = file
            .content_disposition()
            .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;

        let tmp = fastrand::u16(..);

        // Create the temp. file to work with wile we iter over all the chunks.
        let mut f = fs::File::create(format!("./uploads/{}.tmp", tmp)).await?;

        // Create hasher
        let mut hasher = blake3::Hasher::new();

        // iter over all chunks we get from client.
        while let Some(chunk) = file.next().await {
            let data = chunk?;
            hasher.update(&data);
            f.write_all(&data).await?;
        }

        let hash = hasher.finalize().to_hex();

        // We rename in case something goes wrong.
        fs::rename(
            &format!("./uploads/{}.tmp", tmp),
            &format!("./uploads/{}", hash),
        )
        .await?;

        let ext = Path::new(
            &content
                .get_filename()
                .map_or_else(|| "".to_string(), std::string::ToString::to_string),
        )
        .extension()
        .and_then(OsStr::to_str)
        .map_or_else(|| "".to_string(), std::string::ToString::to_string);

        let id = crate::utils::db::insert(
            crate::utils::db::FileRequest {
                mime: file.content_type().to_string(),
                hash: hash.to_string(),
                ext: ext.clone(),
                ip: request
                    .connection_info()
                    .realip_remote_addr()
                    .unwrap_or("")
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .into(),
            },
            db,
        )
        .await
        .map_err(|e| {
            println!("{}", e);
            actix_web::error::ParseError::Incomplete
        })?;

        let domain = format!(
            "{}://{}",
            request.connection_info().scheme(),
            request.connection_info().host()
        );

        // TODO: insert file into database and return id.

        return Ok(HttpResponse::Ok().json(&UploadResp {
            url: format!(
                "{}/{}.{}",
                domain,
                ENCODER
                    .encode_url(id, 1)
                    .map_err(actix_web::error::ErrorInternalServerError)?,
                ext
            ),
        }));
    }
    Err(error::ErrorBadRequest("no files uploaded"))
}

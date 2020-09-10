use crate::utils::{db, ENCODER};
use actix_multipart::{Field, Multipart};
use actix_web::{error, post, web::Data, HttpRequest, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use sled::Tree;
use std::string::ToString;
use tokio::{
    fs::{rename, File},
    io::AsyncWriteExt,
};

#[derive(Serialize)]
struct UploadResp {
    url: String,
}

#[post("")]
pub async fn upload(
    mut multipart: Multipart,
    db: Data<Tree>,
    request: HttpRequest,
) -> Result<HttpResponse> {
    // Handle multipart upload(s) field
    if let Ok(Some(file)) = multipart.try_next().await {
        let mut file: Field = file;
        let content = file
            .content_disposition()
            .ok_or(actix_web::error::ParseError::Incomplete)?;

        let tmp = fastrand::u16(..);

        // Create the temp. file to work with wile we iter over all the chunks.
        let mut f = File::create(["./uploads/", &tmp.to_string(), ".tmp"].concat()).await?;

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
        rename(
            ["./uploads/", &tmp.to_string(), ".tmp"].concat(),
            ["./uploads/", &hash].concat(),
        )
        .await?;

        let ext = content
            .get_filename()
            .and_then(|f| f.split('.').last())
            .map_or_else(|| "".to_string(), ToString::to_string);

        let id = db::insert(
            db::FileRequest {
                mime: file.content_type().to_string(),
                hash: hash.to_string(),
                ext: &ext,
                ip: request
                    .connection_info()
                    .realip_remote_addr()
                    .and_then(|f| f.split(':').next())
                    .map_or_else(|| "".to_string(), ToString::to_string),
            },
            db,
        )
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

        return Ok(HttpResponse::Ok().json(UploadResp {
            url: [
                request.connection_info().scheme(),
                "://",
                request.connection_info().host(),
                "/",
                &ENCODER
                    .encode_url(id, 1)
                    .map_err(actix_web::error::ErrorInternalServerError)?,
                ".",
                &ext,
            ]
            .concat(),
        }));
    }
    Err(error::ErrorBadRequest("no files uploaded"))
}

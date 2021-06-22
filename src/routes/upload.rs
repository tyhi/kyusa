use crate::utils::{db, db::SLED, ENCODER};
use actix_multipart::Multipart;
use actix_web::{error, post, HttpRequest, HttpResponse, Result};
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
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
pub async fn upload(mut multipart: Multipart, request: HttpRequest) -> Result<HttpResponse> {
    // Handle multipart upload(s) field
    if let Ok(Some(mut file)) = multipart.try_next().await {
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

        let ext = file
            .content_disposition()
            .ok_or(actix_web::error::ParseError::Incomplete)?
            .get_filename()
            .and_then(|f| f.split('.').last())
            .map_or_else(|| "".to_string(), ToString::to_string);

        return match file.content_type().type_() {
            mime::IMAGE | mime::VIDEO | mime::AUDIO => {
                let mut df = db::File {
                    id: None,
                    deleted: false,
                    mime: file.content_type().to_string(),
                    hash: hash.to_string(),
                    ext,
                    ip: request
                        .connection_info()
                        .realip_remote_addr()
                        .and_then(|f| f.split(':').next())
                        .map_or_else(|| "".to_string(), ToString::to_string),
                };
                let id = SLED
                    .insert(&mut df)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                Ok(HttpResponse::Ok().json(UploadResp {
                    url: [
                        request.connection_info().scheme(),
                        "://",
                        request.connection_info().host(),
                        "/",
                        &ENCODER
                            .encode_url(id.ok_or(actix_web::error::ParseError::Incomplete)?, 1)
                            .map_err(actix_web::error::ErrorInternalServerError)?,
                        ".",
                        &df.ext,
                    ]
                    .concat(),
                }))
            },
            _ => {
                let mut df = db::File {
                    id: None,
                    deleted: false,
                    mime: file.content_type().to_string(),
                    hash: hash.to_string(),
                    ext,
                    ip: request
                        .connection_info()
                        .realip_remote_addr()
                        .and_then(|f| f.split(':').next())
                        .map_or_else(|| "".to_string(), ToString::to_string),
                };

                let id = SLED
                    .insert(&mut df)
                    .await
                    .map_err(actix_web::error::ErrorInternalServerError)?;

                Ok(HttpResponse::Ok().json(UploadResp {
                    url: [
                        request.connection_info().scheme(),
                        "://",
                        request.connection_info().host(),
                        "/t/",
                        &ENCODER
                            .encode_url(id.unwrap(), 1)
                            .map_err(actix_web::error::ErrorInternalServerError)?,
                        ".",
                        &df.ext,
                    ]
                    .concat(),
                }))
            },
        };
    }

    Err(error::ErrorNotImplemented("err"))
}

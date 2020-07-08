use crate::utils::ENCODER;
use actix_multipart::{Field, Multipart};
use actix_web::{
    error, http::header::ContentDisposition, post, web::Data, HttpRequest, HttpResponse, Result,
};
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
                    .remote()
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
            url: format!("{}/{}.{}", domain, ENCODER.encode_url(id as usize, 5), ext),
        }));
    }
    Err(error::ErrorBadRequest("no files uploaded"))
}

// Takes the input file name and generates the correct paths needed.
async fn gen_upload_file(content: &ContentDisposition) -> anyhow::Result<NamedReturn> {
    let filen = str::replace(
        content
            .get_filename()
            .ok_or_else(|| anyhow::anyhow!("error getting filename"))?,
        " ",
        "_",
    );

    let path = std::path::Path::new(&filen);

    let file_name = path
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| anyhow::anyhow!("no file_name"))?;

    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| anyhow::anyhow!("no extension"))?
        .to_ascii_lowercase();

    // This loop makes sure that we don't have collision in file names.
    loop {
        // Cheaking to see if our file needs to have random name.

        let name = if RANDOM_FILE_EXT.contains(&extension.as_str()) {
            // TODO: placeholder.
            String::new()
        } else {
            file_name.to_owned()
        };

        // Creating our random folder name.
        let folder_dir = String::new();

        let path = format!("./uploads/{}/{}.{}", &folder_dir, &name, &extension);

        if !std::path::Path::new(&path).exists() {
            if !std::path::Path::new(&format!("./uploads/{}", &folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", &folder_dir)).await?;
            }

            return Ok(NamedReturn {
                new_path: path,
                temp_path: format!("./uploads/{}/{}.{}.~tmp", &folder_dir, &name, &extension),
                uri: format!("/{}/{}", &folder_dir, &name),
                ext: extension,
            });
        }
    }
}

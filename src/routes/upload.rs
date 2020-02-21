use crate::{
    routes::delete::del_file,
    utils::{database, models},
    Settings,
};
use actix_multipart::Multipart;
use actix_web::{
    error,
    http::{header::ContentDisposition, HeaderMap},
    post,
    web::Data,
    HttpRequest, HttpResponse, Result,
};
use async_std::prelude::*;
use futures::StreamExt;
use serde::Serialize;
use sqlx::PgPool;
use std::path::Path;

#[derive(Serialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

struct NamedReturn {
    new_path: String,
    temp_path: String,
    uri: String,
    ext: String,
}

const RANDOM_FILE_EXT: &[&str] = &["png", "jpeg", "jpg", "webm", "gif", "avi", "mp4"];

#[post("/u")]
pub async fn upload(
    mut multipart: Multipart,
    config: Data<Settings>,
    request: HttpRequest,
    p: Data<PgPool>,
) -> Result<HttpResponse> {
    let user = check_header(request.headers(), p.clone())
        .await
        .map_err(error::ErrorUnauthorized)?;

    // Handle multipart upload(s)
    while let Some(item) = multipart.next().await {
        match item {
            Err(_) => (),
            Ok(mut file) => {
                let content = file
                    .content_disposition()
                    .ok_or_else(|| actix_web::error::ParseError::Incomplete)?;

                if !check_name(&content, config.multipart_name.as_str())
                    .map_err(error::ErrorInternalServerError)?
                {
                    continue;
                }

                let file_names = gen_upload_file(&content).await.map_err(|err| {
                    error::ErrorInternalServerError(format!("error generating filename: {}", err))
                })?;

                // Create the temp. file to work with wile we iter over all the chunks.
                let mut f = async_std::fs::File::create(&file_names.temp_path).await?;

                // fs keeps track of how big the file is.
                let mut fs = 0;

                // iter over all chunks we get from client.
                while let Some(chunk) = file.next().await {
                    let data = chunk?;
                    f.write_all(&data).await?;
                    fs += data.len();

                    // Hard coded 90MB upload limit to play nice with cloudflare.
                    // Actual limit is 100MB however we might not be able to catch it before a chunk
                    // might put it over the limit.
                    if fs > 90_000_000 {
                        if let Err(err) = del_file(Path::new(&file_names.temp_path)) {
                            return Err(error::ErrorInternalServerError(format!(
                                "file larger than 90MB & failed to clean temp file: {}",
                                err
                            )));
                        }
                        return Err(error::ErrorPayloadTooLarge("larger than 100mb limit"));
                    }
                }

                // Generates the delete key.
                let del_key = nanoid::nanoid!(12, &nanoid::alphabet::SAFE);

                // We rename in case something goes wrong.
                std::fs::rename(&file_names.temp_path, &file_names.new_path)?;
                let domain = format!(
                    "{}://{}",
                    request.connection_info().scheme(),
                    request.connection_info().host()
                );

                database::insert_file(
                    p,
                    models::InsertFile {
                        owner: user.username,
                        uploaded: chrono::Utc::now().naive_utc(),
                        path: format!("{}.{}", file_names.uri, file_names.ext),
                        deletekey: del_key.clone(),
                        filesize: (fs as f64 / 1_000_000.0),
                        downloads: 0,
                    },
                )
                .await
                .map_err(error::ErrorInternalServerError)?;

                return Ok(HttpResponse::Ok().json(&UploadResp {
                    url: format!("{}/u{}.{}", domain, file_names.uri, file_names.ext),
                    delete_url: format!(
                        "{}/d{}.{}?del={}",
                        domain, file_names.uri, file_names.ext, del_key
                    ),
                }));
            },
        }
    }
    Err(error::ErrorBadRequest("no files uploaded"))
}

// Takes the input file name and generates the correct paths needed.
async fn gen_upload_file(
    content: &ContentDisposition,
) -> Result<NamedReturn, Box<dyn std::error::Error>> {
    let path = std::path::Path::new(
        content
            .get_filename()
            .ok_or_else(|| "error getting filename")?,
    );

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "no file_name")?;

    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "no extension")?;

    // This loop makes sure that we don't have collision in file names.
    loop {
        // Cheaking to see if our file needs to have random name.
        let name = match RANDOM_FILE_EXT.iter().any(|x| x == &extension) {
            false => file_name.to_owned(),
            true => nanoid::nanoid!(6, &nanoid::alphabet::SAFE),
        };

        // Creating our random folder name.
        let folder_dir = nanoid::nanoid!(3, &nanoid::alphabet::SAFE);

        let path = format!("./uploads/{}/{}.{}", folder_dir, name, extension);

        if !Path::new(&path).exists() {
            if !Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                async_std::fs::create_dir_all(format!("./uploads/{}", folder_dir)).await?;
            }

            return Ok(NamedReturn {
                new_path: path,
                temp_path: format!("./uploads/{}/{}.{}.~tmp", folder_dir, name, extension),
                uri: format!("/{}/{}", folder_dir, name),
                ext: format!("{}", extension),
            });
        }
    }
}

// check_name checks to make sure we have a multipart name.
fn check_name(field: &ContentDisposition, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    if field
        .get_name()
        .ok_or_else(|| "error getting multipart name")?
        != name
    {
        return Ok(false);
    }
    Ok(true)
}

// Checks headers to see if key is valid.
async fn check_header(
    header: &HeaderMap,
    p: Data<PgPool>,
) -> Result<models::User, Box<dyn std::error::Error>> {
    let apikey = header
        .get("apikey")
        .ok_or_else(|| "apikey header missing")?
        .to_str()?
        .to_string();

    if database::check_api(p.clone(), apikey.clone()).await? {
        return Ok(database::get_user(p, apikey).await?);
    }

    Err("invalid api_key".into())
}

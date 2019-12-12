use crate::{utils::{config::Config, database}, routes::delete::del_file, GLOBAL_DB};
use actix_multipart::{Field, Multipart};
use actix_web::{error, http::HeaderMap, post, web::Data, HttpRequest, HttpResponse, Result};
use futures::StreamExt;
use serde::Serialize;
use std::{fs, io::Write, path::Path};

#[derive(Serialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

struct NamedReturn {
    new_path: String,
    temp_path: String,
    uri: String,
    ffn: String,
    ext: String,
}

const RANDOM_FILE_EXT: &'static [&str] = &["png", "jpeg", "jpg", "webm", "gif", "avi", "mp4"];

#[post("/u")]
pub async fn upload(
    mut multipart: Multipart,
    config: Data<Config>,
    request: HttpRequest,
) -> Result<HttpResponse, error::Error> {
    if config.private {
        if let Err(why) = check_header(request.headers(), &config) {
            return Err(error::ErrorUnauthorized(why));
        }
    }

    while let Some(item) = multipart.next().await {
        let mut field = item?;

        match check_name(&field) {
            Ok(valid) => {
                if !valid {
                    continue;
                }
            },
            Err(e) => return Err(error::ErrorInternalServerError(e)),
        };
        let file_names = match gen_upload_file(&field) {
            Ok(file_names) => file_names,
            Err(err) => {
                return Err(error::ErrorInternalServerError(format!(
                    "error generating filename: {}",
                    err
                )))
            },
        };
        let mut f = std::fs::File::create(&file_names.temp_path)?;
        let mut fs = 0;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            let mut pos = 0;
            while pos < data.len() {
                let bytes_written = f.write(&data[pos..])?;
                pos += bytes_written;
            }
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

        let del_key = nanoid::simple();

        let ins = match database::generate_insert_binary(&file_names.new_path, &del_key) {
            Ok(x) => x,
            Err(err) => return Err(error::ErrorInternalServerError(err)),
        };

        if let Err(err) = GLOBAL_DB.insert(&file_names.ffn.into_bytes(), ins) {
            return Err(error::ErrorInternalServerError(err));
        }

        std::fs::rename(&file_names.temp_path, &file_names.new_path)?;

        return Ok(HttpResponse::Ok().json(&UploadResp {
            url: format!(
                "{}://{}{}.{}",
                config.https, config.domain, file_names.uri, file_names.ext
            ),
            delete_url: format!(
                "{}://{}/d{}.{}?del={}",
                config.https, config.domain, file_names.uri, file_names.ext, del_key
            ),
        }));
    }
    Err(error::ErrorBadRequest("no files uploaded"))
}

fn gen_upload_file(field: &Field) -> Result<NamedReturn, Box<dyn std::error::Error>> {
    let content = field.content_disposition().ok_or("error getting content")?;
    let path = std::path::Path::new(content.get_filename().ok_or("error getting filename")?);

    let file_name = path
        .file_stem()
        .ok_or("missing filename")?
        .to_str()
        .ok_or("error converting to str")?;
    let ext = path
        .extension()
        .ok_or("unable to get ext")?
        .to_str()
        .ok_or("unable to convert to str")?;

    loop {
        let name = match RANDOM_FILE_EXT.iter().any(|x| x == &ext) {
            false => file_name.to_owned(),
            true => nanoid::generate(6),
        };

        let folder_dir = nanoid::generate(2);

        let path = format!("./uploads/{}/{}.{}", folder_dir, name, ext);

        if !Path::new(&path).exists() {
            if !Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", folder_dir))?;
            }

            return Ok(NamedReturn {
                new_path: path,
                temp_path: format!("./uploads/{}/{}.{}.~tmp", folder_dir, name, ext),
                uri: format!("/{}/{}", folder_dir, name),
                ffn: format!("{}{}.{}", folder_dir, name, ext),
                ext: format!("{}", ext),
            });
        }
    }
}

fn check_name(field: &Field) -> Result<bool, Box<dyn std::error::Error>> {
    if field
        .content_disposition()
        .ok_or("error getting disposition")?
        .get_name()
        .ok_or("error getting multipart name")?
        != "file"
    {
        return Ok(false);
    }
    return Ok(true);
}

fn check_header(
    header: &HeaderMap,
    config: &Data<Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    if config
        .key_details
        .get(
            header
                .get("apikey")
                .ok_or("apikey header missing")?
                .to_str()?,
        )
        .is_none()
    {
        return Err("invalid api key".into());
    }
    Ok(())
}

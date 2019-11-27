use crate::{cfg, dbu};
use actix_multipart::{Field, Multipart};
use actix_web::http::HeaderMap;
use actix_web::{error, post, web, Error, HttpRequest, HttpResponse};
use futures::StreamExt;
use serde::Serialize;
use std::fs;
use std::io::Write;

#[derive(Serialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

struct NamedReturn {
    new_path: String,
    uri: String,
    ffn: String,
    ext: String,
}

const RANDOM_FILE_EXT: &'static [&str] = &["png", "jpeg", "jpg", "webm", "gif", "avi", "mp4"];

#[post("/u")]
pub async fn upload(
    mut multipart: Multipart,
    database: web::Data<sled::Db>,
    settings: web::Data<cfg::Config>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    if settings.private {
        match check_header(request.headers(), &settings) {
            Ok(()) => (),
            Err(err) => return Err(error::ErrorUnauthorized(err)),
        }
    }

    while let Some(item) = multipart.next().await {
        let mut field = item?;

        match check_name(&field) {
            Ok(b) => {
                if b == false {
                    continue;
                }
            }
            Err(e) => return Err(error::ErrorInternalServerError(e)),
        };

        let file_names = match gen_upload_file(&field) {
            Ok(file_names) => file_names,
            Err(err) => {
                return Err(error::ErrorInternalServerError(format!(
                    "error generating filename: {}",
                    err
                )))
            }
        };
        let mut f = std::fs::File::create(&file_names.new_path)?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            let mut pos = 0;
            while pos < data.len() {
                let bytes_written = f.write(&data[pos..])?;
                pos += bytes_written;
            }
        }

        let del_key = nanoid::simple();

        let ins = match dbu::generate_insert_binary(&file_names.new_path, &del_key) {
            Ok(x) => x,
            Err(err) => return Err(error::ErrorInternalServerError(err)),
        };

        match database.insert(&file_names.ffn.into_bytes(), ins) {
            Ok(x) => x,
            Err(err) => return Err(error::ErrorInternalServerError(err)),
        };

        return Ok(HttpResponse::Ok().json(&UploadResp {
            url: format!(
                "{}://{}{}.{}",
                settings.https, settings.domain, file_names.uri, file_names.ext
            ),
            delete_url: format!(
                "{}://{}/d{}.{}?del={}",
                settings.https, settings.domain, file_names.uri, file_names.ext, del_key
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

        if !std::path::Path::new(&path).exists() {
            if !std::path::Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", folder_dir))?;
            }

            return Ok(NamedReturn {
                new_path: path,
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
    settings: &web::Data<cfg::Config>,
) -> Result<(), Box<dyn std::error::Error>> {
    if settings
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

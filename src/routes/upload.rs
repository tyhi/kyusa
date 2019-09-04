use crate::dbu;
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use serde::{Serialize};
use std::{fs, ops::Deref};

use crate::ServerSettings;

#[derive(Serialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

struct NamedReturn {
    new_path: String,
    uri: String,
    ffn: String,
}

const RANDOM_FILE_EXT: &'static [&str] = &["png", "jpeg", "webm", "gif", "avi", "mp4"];

pub fn upload(
    mut parts: awmp::Parts,
    database: web::Data<sled::Db>,
    settings: web::Data<ServerSettings>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let file_parts = match parts.files.remove("file").pop() {
        Some(e) => e.persist("./uploads").ok().unwrap(),
        None => {
            return Err(error::ErrorBadRequest(
                "no file was included with multipart post",
            ))
        }
    };

    let ext = match file_parts.extension().unwrap().to_str() {
        Some(e) => e.to_lowercase(),
        None => {
            return Err(error::ErrorBadRequest(
                "files without extensions are not allowed",
            ))
        }
    };

    let filename = file_parts.file_stem().unwrap().to_str().unwrap();

    let file_names = gen_upload_file(filename, &ext);

    fs::rename(file_parts.display().to_string(), &file_names.new_path).unwrap();

    let del_key = nanoid::simple();

    let ins = dbu::generate_insert_binary(
        &file_names.new_path,
        &del_key,
        &request.connection_info().deref(),
    )
    .unwrap();

    database.insert(&file_names.ffn.into_bytes(), ins).unwrap();

    Ok(HttpResponse::Ok().json(&UploadResp {
        url: format!(
            "https://{}{}.{}",
            settings.website_name, file_names.uri, ext
        ),
        delete_url: format!(
            "https://{}/d{}.{}?del={}",
            settings.website_name, file_names.uri, ext, del_key
        ),
    }))
}

fn gen_upload_file(file_name: &str, ext: &String) -> NamedReturn {
    loop {
        let name = match RANDOM_FILE_EXT.iter().any(|x| x == &ext.as_str()) {
            false => file_name.to_owned(),
            true => nanoid::generate(6),
        };

        let folder_dir = nanoid::generate(2);

        let path = format!("./uploads/{}/{}.{}", folder_dir, name, ext);

        if !std::path::Path::new(&path).exists() {
            if !std::path::Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", folder_dir)).unwrap();
            }

            return NamedReturn {
                new_path: path,
                uri: format!("/{}/{}", folder_dir, name),
                ffn: format!("{}{}.{}", folder_dir, name, ext),
            };
        }
    }
}

use crate::dbu;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

pub struct ServerSettings {
    pub api_keys: Vec<String>,
    pub admin_keys: Vec<String>,
    pub website_name: String,
    pub https: bool,
}

pub fn upload(
    mut parts: awmp::Parts,
    database: web::Data<sled::Db>,
    settings: web::Data<ServerSettings>,
    request: HttpRequest,
) -> Result<HttpResponse, Error> {
    let file_parts = parts
        .files
        .remove("file")
        .pop()
        .and_then(|f| f.persist("./uploads").ok())
        .unwrap_or_default();

    let (new_path, uri, ffn) = loop {
        let file_name = nanoid::generate(6);
        let folder_dir = nanoid::generate(2);

        let p = format!(
            "./uploads/{}/{}.{}",
            folder_dir,
            file_name,
            file_parts.extension().unwrap().to_str().unwrap()
        );

        if !std::path::Path::new(&p).exists() {
            if !std::path::Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", folder_dir)).unwrap();
            }
            break (
                p,
                format!("/{}/{}", folder_dir, file_name),
                format!(
                    "{}{}.{}",
                    folder_dir,
                    file_name,
                    file_parts.extension().unwrap().to_str().unwrap()
                ),
            );
        }
    };

    fs::rename(file_parts.display().to_string(), new_path.clone()).unwrap();

    let del_key = nanoid::simple();

    let ins =
        dbu::generate_insert_binary(new_path, del_key.clone(), request.connection_info().deref())
            .unwrap();

    database.insert(ffn.clone().into_bytes(), ins).unwrap();

    let resp_json = UploadResp {
        url: format!(
            "https://{}{}.{}",
            settings.website_name,
            uri,
            file_parts.extension().unwrap().to_str().unwrap()
        ),
        delete_url: format!(
            "https://{}/d{}.{}?del={}",
            settings.website_name,
            uri,
            file_parts.extension().unwrap().to_str().unwrap(),
            del_key
        ),
    };

    Ok(HttpResponse::Ok().json(&resp_json))
}

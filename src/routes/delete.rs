use crate::{cf_file_purge, cfg, dbu};
use actix_web::{error, web, Error, HttpResponse};
use serde::Deserialize;
use std::{fs, path};

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}
#[derive(Deserialize)]
pub struct DeleteFile {
    pub del: String,
}

pub fn delete(
    path: web::Path<FilePath>,
    del: web::Query<DeleteFile>,
    database: web::Data<sled::Db>,
    settings: web::Data<cfg::Config>,
) -> Result<HttpResponse, Error> {
    let binc = match database
        .get(format!("{}{}", path.folder, path.file))
        .unwrap()
    {
        Some(binary) => binary,
        None => {
            return Err(error::ErrorNotFound("this file does not exist"));
        }
    };

    let data: dbu::FileMetadata = bincode::deserialize(&binc[..]).unwrap();

    if del.del != data.del_key {
        return Err(error::ErrorUnauthorized("invalid delete key"));
    }

    database
        .remove(format!("{}{}", path.folder, path.file).into_bytes())
        .unwrap();

    let file_path = path::Path::new(&data.file_path);

    fs::remove_file(file_path)?;

    if file_path
        .parent()
        .unwrap()
        .read_dir()
        .unwrap()
        .next()
        .is_none()
    {
        fs::remove_dir(file_path.parent().unwrap())?;
    }

    if settings.cf_enabled == true {
        let url = format!(
            "{}://{}/{}/{}",
            settings.http_str, settings.domain, path.folder, path.file
        );
        if cf_file_purge::purge_file(
            &settings.cf_zone.as_ref().unwrap(),
            &url,
            &settings.cf_api.as_ref().unwrap(),
        ) != http::StatusCode::OK
        {
            return Err(error::ErrorInternalServerError("file has been delete from os, however there was an error purging cache from cloudflare make sure your key has permission"));
        }
    }
    Ok(HttpResponse::Ok().body("file deleted"))
}

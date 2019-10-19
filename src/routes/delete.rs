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
    let binc = match match database.get(format!("{}{}", path.folder, path.file)) {
        Ok(x) => x,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    } {
        Some(binary) => binary,
        None => {
            return Err(error::ErrorNotFound("this file does not exist"));
        }
    };

    let data: dbu::FileMetadata = match bincode::deserialize(&binc[..]) {
        Ok(x) => x,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    if del.del != data.del_key {
        return Err(error::ErrorUnauthorized("invalid delete key"));
    }

    match database.remove(format!("{}{}", path.folder, path.file).into_bytes()) {
        Ok(x) => x,
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    };

    let file_path = path::Path::new(&data.file_path);

    match del_file(file_path) {
        Ok(_) => (),
        Err(err) => return Err(error::ErrorInternalServerError(err)),
    }

    if settings.cloudflare_details.is_some() == true {
        let url = format!(
            "{}://{}/{}/{}",
            settings.http_str, settings.domain, path.folder, path.file
        );
        match cf_file_purge::purge_file(
            &settings.cloudflare_details.as_ref().unwrap().cf_zone,
            &url,
            &settings.cloudflare_details.as_ref().unwrap().cf_api,
        ) {
            Ok(status) => {
                if status != http::StatusCode::OK {
                    return Err(error::ErrorInternalServerError("file has been delete from os, however there was an error purging cache from cloudflare make sure your key has permission"));
                }
            }
            Err(err) => return Err(error::ErrorInternalServerError(err)),
        }
    }
    Ok(HttpResponse::Ok().body("file deleted"))
}

fn del_file(file_path: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(file_path)?;

    // The unwraps here should never fault because it's impossible with our setup to never have a parent dir.
    if match file_path.parent() {
        Some(x) => x,
        None => unimplemented!(),
    }
    .read_dir()?
    .next()
    .is_none()
    {
        fs::remove_dir(match file_path.parent() {
            Some(x) => x,
            None => unimplemented!(),
        })?;
    };
    Ok(())
}

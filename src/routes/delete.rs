use crate::{cf_file_purge, dbu, ServerSettings};
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
    settings: web::Data<ServerSettings>,
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
        fs::remove_dir(file_path.parent().unwrap()).unwrap();
    }

    if settings.cloudflare.enabled == true {
        let url = format!(
            "https://{}/{}/{}",
            settings.website_name, path.folder, path.file
        );

        cf_file_purge::purge_file(
            &settings.cloudflare.zone.clone().unwrap(),
            &url,
            &settings.cloudflare.cf_key,
        )
        .unwrap();
    }
    Ok(HttpResponse::Ok().body("file deleted"))
}

use crate::utils::{config::Config, database};
use actix_web::{error, get, web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use sqlx::PgPool;
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
#[get("/d/{folder}/{file}")]
pub async fn delete(
    config: web::Data<Config>,
    path: web::Path<FilePath>,
    del: web::Query<DeleteFile>,
    p: web::Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse> {
    let file = database::get_file(p, format!("/{}/{}", path.folder, path.file))
        .await
        .unwrap();

    if file.deletekey != del.del {
        return Err(error::ErrorUnauthorized("not a valid delete key"));
    }

    // remove from db

    let fp = format!("./uploads{}", file.path);
    let file_path = path::Path::new(&fp);

    if let Err(err) = del_file(file_path) {
        return Err(error::ErrorInternalServerError(err));
    }

    if let Some(cf) = &config.cloudflare_details {
        let url = format!(
            "{}://{}{}",
            request.connection_info().scheme(),
            request.connection_info().host(),
            file.path
        );
        match cfp_rs::purge_file(cf.cf_zone.as_str(), &url, &cf.cf_api.as_str()).await {
            Ok(status) => {
                if status != 200 {
                    return Err(error::ErrorInternalServerError(
                        "file has been delete from os, however there was an error purging cache \
                         from cloudflare make sure your key has permission",
                    ));
                }
            },
            Err(err) => return Err(error::ErrorInternalServerError(err)),
        }
    }

    Ok(HttpResponse::Ok().body("file deleted"))
}

pub fn del_file(file_path: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(file_path)?;

    if file_path
        .parent()
        .ok_or("no parent directory")?
        .read_dir()?
        .next()
        .is_none()
    {
        fs::remove_dir(file_path.parent().ok_or("no parent directory")?)?;
    }
    Ok(())
}

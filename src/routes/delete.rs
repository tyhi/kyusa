use crate::{
    utils::{cf, db},
    Settings,
};
use actix_web::{
    error, get,
    web::{Data, Path},
    HttpRequest, HttpResponse, Result,
};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
pub struct Key {
    pub key: String,
}

#[get("/d/{key}")]
pub async fn delete(
    config: Data<Settings>,
    path: Path<Key>,
    db: Data<sled::Db>,
    request: HttpRequest,
) -> Result<HttpResponse> {
    let file = db::get_file_by_del(Data::clone(&db), path.key.clone())
        .await
        .map_err(|_| {
            error::ErrorNotFound(
                "file has already been deleted, or it may not have existed in the first place",
            )
        })?;

    del_file(std::path::Path::new(&format!("./uploads{}", file.path)))
        .await
        .map_err(error::ErrorInternalServerError)?;

    db::delete_file(db, file.path.clone())
        .await
        .map_err(error::ErrorInternalServerError)?;

    if let Some(cf) = &config.cloudflare_details {
        let url = format!(
            "{}://{}{}",
            request.connection_info().scheme(),
            request.connection_info().host(),
            file.path
        );

        if let StatusCode::OK = cf::purge(&cf.cloudflare_api, &cf.cloudflare_zone, &url)
            .await
            .map_err(error::ErrorInternalServerError)?
        {
        } else {
            return Err(error::ErrorInternalServerError(
                "file has been delete from os, however there was an error purging cache from \
                 cloudflare make sure your key has permission",
            ));
        }
    }

    Ok(HttpResponse::Ok().body("file deleted"))
}

pub async fn del_file(file_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::remove_file(file_path).await?;

    if file_path
        .parent()
        .ok_or_else(|| "no parent directory")?
        .read_dir()?
        .next()
        .is_none()
    {
        fs::remove_dir(file_path.parent().ok_or_else(|| "no parent directory")?).await?;
    }
    Ok(())
}

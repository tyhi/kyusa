use crate::{
    utils::{cf, database},
    Settings,
};
use actix_web::{
    error, get,
    web::{Data, Path},
    HttpRequest, HttpResponse, Result,
};
use async_std::path;
use futures::StreamExt;
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct Key {
    pub key: String,
}

#[get("/d/{key}")]
pub async fn delete(
    config: Data<Settings>,
    path: Path<Key>,
    p: Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse> {
    let file = database::get_file_by_del(Data::clone(&p), &path.key)
        .await
        .map_err(|_| {
            error::ErrorNotFound(
                "file has already been deleted, or it may not have existed in the first place",
            )
        })?;

    let pa = format!("./uploads{}", file.path);
    let file_path = path::Path::new(&pa);

    del_file(file_path)
        .await
        .map_err(error::ErrorInternalServerError)?;

    database::delete_file(p, &file.path)
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

pub async fn del_file(file_path: &path::Path) -> Result<(), Box<dyn std::error::Error>> {
    async_std::fs::remove_file(file_path).await?;

    if file_path
        .parent()
        .ok_or_else(|| "no parent directory")?
        .read_dir()
        .await?
        .next()
        .await
        .is_none()
    {
        async_std::fs::remove_dir(file_path.parent().ok_or_else(|| "no parent directory")?).await?;
    }
    Ok(())
}

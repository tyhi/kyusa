use actix_web::web::Data;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct File {
    pub id: i64,
    pub hash: String,
    pub ext: String,
    pub ip: String,
    pub mime: String,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct FileRequest {
    pub hash: String,
    pub ext: String,
    pub mime: String,
    pub ip: String,
}

pub async fn insert(rqe: FileRequest, pg: Data<PgPool>) -> Result<i64> {
    let mut tx = pg.begin().await?;

    let e = sqlx::query_as!(File, r#"SELECT * FROM files WHERE hash = $1"#, rqe.hash)
        .fetch_optional(&mut tx)
        .await?;

    if let Some(e) = e {
        if !e.deleted {
            return Ok(e.id);
        }
    }

    let resp = sqlx::query!(
        r#"INSERT INTO files (hash, ext, ip, mime) VALUES ($1, $2, $3, $4) RETURNING id"#,
        rqe.hash,
        rqe.ext,
        rqe.ip,
        rqe.mime
    )
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(resp.id)
}

pub async fn get(id: i64, pg: Data<PgPool>) -> Result<Option<File>> {
    let mut tx = pg.begin().await?;

    let resp = sqlx::query_as!(File, r#"SELECT * FROM files WHERE id = $1"#, id)
        .fetch_optional(&mut tx)
        .await?;

    Ok(resp)
}

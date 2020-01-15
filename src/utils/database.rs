use super::models;
use crate::utils::models::User;
use actix_web::web::Data;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct FileMetadata<'a> {
    pub file_path: &'a str,
    pub del_key: &'a str,
    pub time_date: DateTime<Utc>,
}

// new database stuff below
pub async fn get_user(
    p: Data<PgPool>,
    api: String,
) -> Result<models::User, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let rec: models::User = sqlx::query_as!(
        models::User,
        r#"
            select *
            from kyous.public.users
            where apikey = $1
        "#,
        api
    )
    .fetch_one(&mut tx)
    .await?;

    Ok(rec)
}

pub async fn insert_user(
    p: PgPool,
    user: models::InsertUser,
) -> Result<models::User, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let resp = sqlx::query!(
        r#"
            INSERT INTO users ( username, email, apikey, ipaddr )
            VALUES ( $1, $2, $3, $4 )
            RETURNING *
        "#,
        user.username,
        user.email,
        user.apikey,
        user.ipaddr
    )
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;

    Ok(User {
        id: resp.id,
        username: resp.username,
        email: resp.email,
        ipaddr: resp.ipaddr,
        apikey: resp.apikey,
    })
}

pub async fn insert_file(
    p: Data<PgPool>,
    file: models::InsertFile,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let rec = sqlx::query!(
        r#"
        INSERT INTO files (owner, path, deletekey)
        VALUES ($1, $2, $3)
        RETURNING id
    "#,
        file.owner,
        file.path,
        file.deletekey
    )
    .fetch_one(&mut tx)
    .await
    .unwrap();

    tx.commit().await?;

    Ok(rec.id)
}

pub async fn get_file(
    p: Data<PgPool>,
    path: String,
) -> Result<models::File, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let resp: models::File = sqlx::query_as!(
        models::File,
        r#"
            SELECT *
            FROM files
            WHERE path = $1
        "#,
        path
    )
    .fetch_one(&mut tx)
    .await?;

    Ok(resp)
}

pub async fn check_api(
    p: Data<PgPool>,
    apikey: String,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let resp = sqlx::query!(
        r#"
            SELECT COUNT(*)
            FROM users
            WHERE apikey = $1
        "#,
        apikey
    )
    .fetch_one(&mut tx)
    .await?;
    return if resp.count == 0 { Ok(false) } else { Ok(true) };
}

pub async fn file_count(p: Data<PgPool>) -> Result<i64, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;
    let resp = sqlx::query!(
        r#"
            SELECT COUNT(*)
            from files
        "#
    )
    .fetch_one(&mut tx)
    .await?;

    Ok(resp.count)
}

use super::models;
use crate::utils::models::User;
use actix_web::web::Data;
use sqlx::PgPool;
use uuid::Uuid;

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
            INSERT INTO files (owner, path, deletekey, filesize)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        "#,
        file.owner,
        file.path,
        file.deletekey,
        file.filesize
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
    if resp.count == 0 {
        Ok(false)
    } else {
        Ok(true)
    }
}

pub async fn delete_file(
    p: Data<PgPool>,
    filepath: String,
) -> Result<Uuid, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;
    let id = sqlx::query!(
        r#"
            DELETE FROM files
            WHERE path = $1
            RETURNING id
        "#,
        filepath
    )
    .fetch_one(&mut tx)
    .await?;
    tx.commit().await?;

    Ok(id.id)
}

pub async fn get_metrics(p: Data<PgPool>) -> Result<models::Metrics, Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;
    let metrics: models::Metrics = sqlx::query_as!(
        models::Metrics,
        r#"
            SELECT
            (SELECT coalesce(count(*), 0)::bigint from files as bigint) as files,
            (SELECT coalesce(count(*), 0)::bigint from users) as users,
            (SELECT coalesce(sum(downloads*filesize), 0)::float8 as bigint from files) as served,
            (SELECT coalesce(sum(filesize), 0)::float8 from files as bigint) as stored
        "#
    )
    .fetch_one(&mut tx)
    .await?;

    Ok(metrics)
}

pub async fn inc_file(p: Data<PgPool>, filepath: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = p.begin().await?;

    let _ = sqlx::query!(
        r#"
            UPDATE files
            SET downloads = downloads + 1
            WHERE path = $1
            RETURNING id
        "#,
        filepath
    )
    .fetch_one(&mut tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

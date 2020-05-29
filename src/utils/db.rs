use super::models;
use actix_web::web::Data;
use anyhow::{Context, Result};
use sled::Db;
use tokio::task::spawn_blocking;

pub async fn get_user(db: Data<Db>, api: &str) -> Result<models::User> {
    let key = db
        .get(format!("api_{}", api))?
        .with_context(|| "user does not exist")?;

    Ok(bincode::deserialize::<models::User>(&key)?)
}

pub async fn insert_user(db: Data<Db>, user: models::User) -> Result<()> {
    db.insert(format!("api_{}", user.apikey), bincode::serialize(&user)?)?;
    Ok(())
}

pub async fn insert_file(db: Data<Db>, file: models::File) -> Result<()> {
    db.insert(format!("file_{}", file.path), bincode::serialize(&file)?)?;
    Ok(())
}

pub async fn get_file(db: Data<Db>, path: String) -> Result<models::File> {
    let file = db
        .get(format!("file_{}", path))?
        .with_context(|| "file not found")?;
    Ok(bincode::deserialize::<models::File>(&file)?)
}

pub async fn get_file_by_del(db: Data<Db>, key: String) -> Result<models::File> {
    for i in db.scan_prefix("file_") {
        let (_, data) = i?;
        let e = bincode::deserialize::<models::File>(&data)?;
        if e.deletekey == key {
            return Ok(e);
        }
    }
    Err(anyhow::anyhow!("file with key not found"))
}

pub async fn check_api(db: Data<Db>, key: &str) -> Result<bool> {
    Ok(db.get(format!("api_{}", key))?.is_some())
}

pub async fn delete_file(db: Data<Db>, path: String) -> Result<()> {
    db.remove(format!("file_{}", path))?;
    Ok(())
}

#[allow(clippy::cast_precision_loss, clippy::as_conversions)]
pub async fn get_metrics(db: Data<Db>) -> Result<models::Metrics> {
    let mut metrics: models::Metrics = models::Metrics {
        files: 0,
        users: 0,
        served: 0.0,
        stored: 0.0,
    };

    for file in db.scan_prefix("file_") {
        let (_, file) = file?;
        let file = bincode::deserialize::<models::File>(&file)?;
        metrics.files += 1;
        metrics.stored += &file.filesize;
        metrics.served += file.filesize * file.downloads as f64;
    }
    for user in db.scan_prefix("api_") {
        user?;
        metrics.users += 1;
    }
    Ok(metrics)
}

pub async fn inc_file(db: Data<Db>, path: String) -> Result<()> {
    db.fetch_and_update(format!("file_{}", path), increment)?;
    Ok(())
}

fn increment(old: Option<&[u8]>) -> Option<Vec<u8>> {
    match old {
        Some(bytes) => {
            let mut file = bincode::deserialize::<models::File>(bytes).expect(
                "FATAL ERROR: this should never happen. If it did please contact project owner.",
            );
            file.downloads += 1;
            Some(bincode::serialize(&file).expect(
                "FATAL ERROR: this should never happen. If it did please contact project owner.",
            ))
        },
        None => None,
    }
}

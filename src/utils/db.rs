use actix_web::web::Data;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sled::{IVec, Tree};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
pub struct FileRequest<'a> {
    pub hash: String,
    pub ext: &'a str,
    pub mime: String,
    pub ip: String,
}

pub async fn insert(rqe: FileRequest<'_>, db: Data<Tree>) -> Result<i64> {
    if let Some(file) = get_hash(&rqe.hash, &db).await {
        return Ok(file.id);
    }

    let id = (db.len() + 1) as i64;

    db.insert(
        (db.len() + 1).to_be_bytes(),
        bin(File {
            id,
            hash: rqe.hash,
            ext: rqe.ext.to_string(),
            ip: rqe.ip,
            mime: rqe.mime,
            deleted: false,
        })?,
    )?;

    Ok(id)
}

pub async fn get_hash(hash: &str, db: &Data<Tree>) -> Option<File> {
    for k in db.iter() {
        if let Ok(file) = debin::<File>(&k.ok()?.1) {
            if file.hash == hash {
                return Some(file);
            }
        }
    }

    None
}

fn debin<T: DeserializeOwned>(i: &IVec) -> Result<T> { Ok(bincode::deserialize::<T>(i)?) }
fn bin<T: Serialize>(s: T) -> Result<Vec<u8>> { Ok(bincode::serialize(&s)?) }

pub async fn get(id: i64, pg: Data<Tree>) -> Option<File> {
    if let Ok(Some(file)) = pg.get(id.to_be_bytes()) {
        return Some(debin::<File>(&file).ok()?);
    };

    None
}

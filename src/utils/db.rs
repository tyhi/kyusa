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
pub struct FileRequest {
    pub hash: String,
    pub ext: String,
    pub mime: String,
    pub ip: String,
}

pub async fn insert(rqe: FileRequest, db: Data<Tree>) -> Result<i64> {
    if let Some(file) = get_hash(&rqe.hash, &db).await {
        return Ok(file.id);
    }

    db.insert(
        (db.len() + 1).to_be_bytes(),
        bin(File {
            id: (db.len() + 1) as i64,
            hash: rqe.hash,
            ext: rqe.ext,
            ip: rqe.ip,
            mime: rqe.mime,
            deleted: false,
        }),
    )
    .unwrap();

    Ok(1)
}

pub async fn get_hash(hash: &str, db: &Data<Tree>) -> Option<File> {
    for k in db.iter() {
        if let Ok(file) = debin::<File>(&k.unwrap().1) {
            if file.hash == hash {
                return Some(file);
            }
        }
    }

    None
}

fn debin<T: DeserializeOwned>(i: &IVec) -> Result<T> { Ok(bincode::deserialize::<T>(i)?) }
fn bin<T: Serialize>(s: T) -> Vec<u8> { bincode::serialize(&s).unwrap() }

pub async fn get(id: i64, pg: Data<Tree>) -> Option<File> {
    if let Ok(Some(file)) = pg.get(id.to_be_bytes()) {
        return Some(debin::<File>(&file).unwrap());
    };

    None
}

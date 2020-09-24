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

#[derive(Serialize, Deserialize)]
pub struct FileTmp {
    pub id: String,
    pub hash: String,
    pub ext: String,
    pub ip: String,
    pub mime: String,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct FileRequestTmp<'a> {
    pub hash: String,
    pub ext: &'a str,
    pub mime: String,
    pub ip: String,
}

pub async fn insert_tmp(rqe: FileRequestTmp<'_>, id: &str, db: Tree) -> Result<String> {
    if let Some(file) = get_hash_tmp(&rqe.hash, &db).await {
        return Ok(file.id);
    }

    db.insert(
        blake3::hash(id.as_bytes()).as_bytes(),
        bin(FileTmp {
            id: id.to_string(),
            hash: rqe.hash,
            ext: rqe.ext.to_string(),
            ip: rqe.ip,
            mime: rqe.mime,
            deleted: false,
        })?,
    )?;

    Ok(id.to_string())
}

pub async fn insert(rqe: FileRequest<'_>, db: Tree) -> Result<i64> {
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

pub async fn get_hash(hash: &str, db: &Tree) -> Option<File> {
    for k in db.iter() {
        if let Ok(file) = debin::<File>(&k.ok()?.1) {
            if file.hash == hash {
                return Some(file);
            }
        }
    }

    None
}

pub async fn get_hash_tmp(hash: &str, db: &Tree) -> Option<FileTmp> {
    for k in db.iter() {
        if let Ok(file) = debin::<FileTmp>(&k.ok()?.1) {
            if file.hash == hash {
                return Some(file);
            }
        }
    }

    None
}

fn debin<T: DeserializeOwned>(i: &IVec) -> Result<T> { Ok(bincode::deserialize::<T>(i)?) }
fn bin<T: Serialize>(s: T) -> Result<Vec<u8>> { Ok(bincode::serialize(&s)?) }

pub async fn get(id: i64, pg: Tree) -> Option<File> {
    if let Ok(Some(file)) = pg.get(id.to_be_bytes()) {
        return Some(debin::<File>(&file).ok()?);
    };

    None
}

pub async fn get_tmp(id: &str, pg: Tree) -> Option<FileTmp> {
    if let Ok(Some(file)) = pg.get(blake3::hash(id.as_bytes()).as_bytes()) {
        return Some(debin::<FileTmp>(&file).ok()?);
    };

    None
}

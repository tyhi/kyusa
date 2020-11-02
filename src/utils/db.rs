use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sled::{Db, IVec};
use std::sync::Arc;

pub static SLED: Lazy<Arc<SledD>> = Lazy::new(|| Arc::new(SledD::new()));

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct SledD {
    db: Db,
}

impl SledD {
    pub fn new() -> Self {
        SledD {
            db: sled::open("./db").unwrap(),
        }
    }

    pub async fn insert(&self, rqe: &mut File) -> Result<Option<i64>> {
        if let Some(file) = self.get_hash(&rqe.hash).await {
            return Ok(file.id);
        }

        let id = (self.db.len() + 1) as i64;

        rqe.id = Some(id);

        self.db
            .insert((self.db.len() + 1).to_be_bytes(), bin(rqe)?)?;

        Ok(Some(id))
    }

    pub async fn get(&self, id: i64) -> Option<File> {
        if let Ok(Some(file)) = self.db.get(id.to_be_bytes()) {
            return Some(debin::<File>(&file).ok()?);
        };

        None
    }

    pub async fn get_hash(&self, hash: &str) -> Option<File> {
        for k in self.db.iter() {
            if let Ok(file) = debin::<File>(&k.ok()?.1) {
                if file.hash == hash {
                    return Some(file);
                }
            }
        }

        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub id: Option<i64>,
    pub hash: String,
    pub ext: String,
    pub ip: String,
    pub mime: String,
    pub deleted: bool,
}

fn debin<T: DeserializeOwned>(i: &IVec) -> Result<T> { Ok(bincode::deserialize::<T>(i)?) }
fn bin<T: Serialize>(s: T) -> Result<Vec<u8>> { Ok(bincode::serialize(&s)?) }

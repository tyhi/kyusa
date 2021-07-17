use once_cell::sync::Lazy;
use rkyv::{
    archived_root,
    ser::{serializers::AllocSerializer, Serializer},
    AlignedVec, Archive, Deserialize, Infallible, Serialize,
};
use sled::{Db, IVec};
use std::sync::Arc;

pub static SLED: Lazy<Arc<SledD>> = Lazy::new(|| Arc::new(SledD::new()));

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct SledD {
    db: Db,
}

impl SledD {
    pub fn new() -> Self {
        Self {
            db: sled::open("./db").unwrap(),
        }
    }

    pub async fn insert(&self, rqe: &File) -> Result<u64> {
        if let Some(file) = self.get_hash(&rqe.hash).await {
            return Ok(file.id);
        }

        self.db.insert(rqe.id.to_be_bytes(), bin(rqe)?.as_slice())?;

        Ok(rqe.id)
    }

    pub async fn get(&self, id: u64) -> Option<File> {
        if let Ok(Some(file)) = self.db.get(id.to_be_bytes()) {
            return debin(&file).ok();
        };

        None
    }

    pub async fn get_hash(&self, hash: &str) -> Option<File> {
        for k in self.db.iter() {
            if let Ok(file) = debin(&k.ok()?.1) {
                if file.hash == hash {
                    return Some(file);
                }
            }
        }
        None
    }

    pub fn get_id(&self) -> u64 { self.db.generate_id().unwrap() }
}

#[derive(Archive, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    pub hash: String,
    pub ext: String,
    pub ip: String,
    pub mime: String,
    pub deleted: bool,
}

fn debin(i: &IVec) -> Result<File> {
    let archived = unsafe { archived_root::<File>(&i[..]) };

    Ok(archived.deserialize(&mut Infallible)?)
}
fn bin(s: &File) -> Result<AlignedVec> {
    let mut ser = AllocSerializer::<256>::default();

    ser.serialize_value(s)?;

    Ok(ser.into_serializer().into_inner())
}

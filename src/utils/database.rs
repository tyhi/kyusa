use crate::GLOBAL_DB;
use bincode::serialize;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileMetadata<'a> {
    pub file_path: &'a str,
    pub del_key: &'a str,
    pub time_date: DateTime<Utc>,
}

pub fn generate_insert_binary(
    file_path: &String,
    del_key: &String,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = FileMetadata {
        file_path,
        del_key,
        time_date: chrono::Utc::now(),
    };
    Ok(serialize(&metadata)?)
}

pub fn get_entry(key: &String) -> Result<sled::IVec, Box<dyn std::error::Error>> {
    Ok(GLOBAL_DB.get(key)?.ok_or("no key found")?)
}

pub fn de_ser<'de, T>(binc: &'de sled::IVec) -> Result<T, Box<dyn std::error::Error>>
where
    T: Deserialize<'de>,
{
    let data: T = bincode::deserialize(&binc[..])?;
    Ok(data)
}

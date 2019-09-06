use actix_web::dev::ConnectionInfo;
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
    filepath: &String,
    delkey: &String,
) -> Result<Vec<u8>, &'static str> {
    let metadata = FileMetadata {
        file_path: filepath,
        del_key: delkey,
        time_date: chrono::Utc::now(),
    };
    Ok(serialize(&metadata).unwrap())
}

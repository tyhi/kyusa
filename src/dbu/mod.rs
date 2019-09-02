use actix_web::dev::ConnectionInfo;
use bincode::serialize;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct FileMetadata<'a> {
    pub ip: &'a str,
    pub file_path: &'a str,
    pub del_key: &'a str,
    pub time_date: DateTime<Utc>,
}

pub fn generate_insert_binary(
    filepath: &String,
    delkey: &String,
    uinfo: &ConnectionInfo,
) -> Result<Vec<u8>, &'static str> {
    let metadata = FileMetadata {
        ip: uinfo.remote().unwrap(),
        file_path: filepath,
        del_key: delkey,
        time_date: chrono::Utc::now(),
    };
    Ok(serialize(&metadata).unwrap())
}

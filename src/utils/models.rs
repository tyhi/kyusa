use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub apikey: String,
    pub ipaddr: String,
}

pub struct InsertUser {
    pub username: String,
    pub email: String,
    pub apikey: String,
    pub ipaddr: String,
}

#[derive(Serialize, Debug)]
pub struct File {
    pub id: Uuid,
    pub owner: String,
    pub uploaded: NaiveDateTime,
    pub path: String,
    pub deletekey: String,
    pub filesize: i64,
    pub downloads: i64,
}

pub struct InsertFile {
    pub owner: String,
    pub uploaded: NaiveDateTime,
    pub path: String,
    pub deletekey: String,
    pub filesize: i64,
    pub downloads: i64,
}

pub struct Metrics {
    pub files: i64,
    pub users: i64,
    pub served: i64,
    pub stored: i64,
}

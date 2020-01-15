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
}

pub struct InsertFile {
    pub owner: String,
    pub uploaded: NaiveDateTime,
    pub path: String,
    pub deletekey: String,
}

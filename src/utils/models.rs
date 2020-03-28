use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
    pub email: String,
    pub apikey: String,
    pub ipaddr: String,
    pub admin: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub owner: String,
    pub uploaded: NaiveDateTime,
    pub path: String,
    pub deletekey: String,
    pub filesize: f64,
    pub downloads: i64,
}

pub struct Metrics {
    pub files: i64,
    pub users: i64,
    pub served: f64,
    pub stored: f64,
}

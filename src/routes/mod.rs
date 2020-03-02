use actix_web::{get, web, Result, Scope};
use std::str::from_utf8;

pub mod delete;
pub mod serve;
pub mod stats;
pub mod upload;
pub mod users;

static MSG: &[u8] = &[
    100, 101, 97, 114, 32, 107, 105, 114, 115, 116, 101, 110, 44, 10, 10, 105, 32, 108, 111, 118,
    101, 32, 121, 111, 117, 10, 10, 121, 111, 117, 114, 115, 32, 97, 108, 119, 97, 121, 115, 44,
    10, 116, 121, 108, 101, 114,
];

#[get("/k")]
pub async fn k() -> Result<&'static str> { Ok(from_utf8(MSG)?) }

#[get("")]
pub async fn index() -> &'static str { "index" }

pub fn routes() -> Scope {
    web::scope("/")
        .service(index)
        .service(delete::delete)
        .service(stats::stats)
        .service(upload::upload)
        .service(users::register)
        .service(users::del_user)
        .service(users::get_user_stats)
        .service(k)
        .service(serve::serve)
}

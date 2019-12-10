use actix_web::{get, web, Result, Scope};
use std::str::from_utf8;

pub mod delete;
pub mod serve;
pub mod stats;
pub mod upload;

static MSG: &[u8] = &[
    105, 32, 108, 111, 118, 101, 32, 107, 105, 114, 115, 116, 101, 110,
];

#[get("/k")]
pub async fn k() -> Result<&'static str> { Ok(from_utf8(MSG)?) }

pub fn routes() -> Scope {
    web::scope("/")
        .service(delete::delete)
        .service(serve::serve)
        .service(stats::stats)
        .service(upload::upload)
        .service(k)
}

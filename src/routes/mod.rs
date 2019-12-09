use actix_web::{web, Scope};

pub mod delete;
pub mod serve;
pub mod stats;
pub mod upload;

pub fn routes() -> Scope {
    web::scope("/")
        .service(delete::delete)
        .service(serve::serve)
        .service(stats::stats)
        .service(upload::upload)
}

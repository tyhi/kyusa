use actix_web::{get, web, Scope};

pub mod serve;
pub mod upload;

#[get("")]
#[allow(clippy::unused_async)]
pub async fn index() -> &'static str {
    "welcome to kyusa, a fast file upload server built in rust."
}

pub fn routes() -> Scope {
    web::scope("/")
        .service(index)
        .service(upload::upload)
        .service(serve::serve)
}

use actix_web::{get, web, Scope};

pub mod serve;
pub mod upload;

#[get("")]
pub async fn index() -> &'static str {
    "welcome to kyusa, a fast file upload server built in rust. \n\nfile uploads limited to 95MB \
     to allow us to use cloudflare cdn to our server. upload in parts if needed.\n\n\ncurrently \
     you must have an account to upload but this will change. \nin the future free uploads will \
     have smaller file retention and registered will have longer"
}

pub fn routes() -> Scope {
    web::scope("/")
        .service(index)
        .service(upload::upload)
        .service(serve::serve)
}

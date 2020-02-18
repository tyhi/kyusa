use actix_web::web;
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

mod routes;
mod utils;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Clone)]
pub struct Settings {
    pub multipart_name: String,
    pub cloudflare_details: Option<Cloudflare>,
}

#[derive(Clone)]
pub struct Cloudflare {
    pub cloudflare_api: String,
    pub cloudflare_zone: String,
}

async fn p404() -> &'static str { "this resource does not exist." }

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let pool = PgPool::new(&env::var("DATABASE_URL")?).await?;

    let cloudflare: Option<Cloudflare>;

    if env::var("KYUSA_CLOUDFLARE")? == "true" {
        cloudflare = Some(Cloudflare {
            cloudflare_zone: env::var("")?,
            cloudflare_api: env::var("")?,
        })
    } else {
        cloudflare = None;
    }

    let settings = Settings {
        multipart_name: env::var("KYUSA_MULTIPARTNAME")?,
        cloudflare_details: cloudflare,
    };

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    if !std::path::Path::new("./tmp").exists() {
        std::fs::create_dir_all("./tmp")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(settings.clone())
            .data(pool.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind(format!("0.0.0.0:{}", env::var("KYUSA_PORT")?))?
    .run()
    .await?;
    Ok(())
}

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::integer_arithmetic,
    clippy::implicit_return,
    clippy::float_arithmetic,
    dead_code
)]

use actix_web::web;
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

mod routes;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
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
            cloudflare_zone: env::var("CLOUDFLARE_ZONE")?,
            cloudflare_api: env::var("CLOUDFLARE_API")?,
        })
    } else {
        cloudflare = None;
    }

    let settings = Settings {
        multipart_name: env::var("KYUSA_MULTIPARTNAME")?,
        cloudflare_details: cloudflare,
    };

    // This is our cron job that will run various tasks every once in a while.
    // However this thread will never quit.
    async_std::task::spawn(
        enclose! { (pool, settings) async move { utils::cron::init(pool, settings).await}},
    );

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

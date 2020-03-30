#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::integer_arithmetic,
    clippy::implicit_return,
    clippy::float_arithmetic,
    clippy::panic,
    clippy::result_expect_used,
    dead_code
)]

use actix_web::web;
use dotenv::dotenv;
use std::env;

mod routes;
mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

    let db = sled::Config::new()
        .use_compression(true)
        .path("./db")
        .open()?;

    db.len();

    first_run_check(web::Data::new(db.clone())).await?;

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

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads")?;
    }

    if !std::path::Path::new("./tmp").exists() {
        std::fs::create_dir_all("./tmp")?;
    }

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(settings.clone())
            .data(db.clone())
            .service(routes::routes())
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind(format!("0.0.0.0:{}", env::var("KYUSA_PORT")?))?
    .run()
    .await?;
    Ok(())
}

#[allow(clippy::print_stdout)]
async fn first_run_check(db: web::Data<sled::Db>) -> anyhow::Result<()> {
    let m = utils::db::get_metrics(db.clone()).await?;
    if m.users == 0 {
        println!("Detected first run, setting up default admin account.");
        println!("Enter username:");
        let username = get_input();

        println!("Enter email:");
        let email = get_input();

        let user = utils::models::User {
            username,
            email,
            apikey: nanoid::nanoid!(24, &nanoid::alphabet::SAFE),
            ipaddr: "NA".to_string(),
            admin: true,
        };

        println!("{}", format!("\nHere is your apikey: {}", user.apikey));

        utils::db::insert_user(db, user).await?;
    }

    Ok(())
}

fn get_input() -> String {
    let mut s = String::new();
    std::io::stdin()
        .read_line(&mut s)
        .expect("error reading stdin");
    s.trim().to_string()
}

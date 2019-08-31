use actix_files::NamedFile;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::fs;
use std::ops::Deref;

mod dbu;

#[derive(Serialize, Deserialize)]
struct FileInfo {
    delete_key: String,
    path: String,
}

#[derive(Serialize, Clone, Deserialize)]
struct ServerSettings {
    api_keys: Vec<String>,
    admin_keys: Vec<String>,
    website_name: String,
    https: bool,
}

#[derive(Deserialize)]
struct FilePath {
    folder: String,
    file: String,
}

#[derive(Deserialize)]
struct DeleteFile {
    del: String,
}

#[derive(Serialize, Deserialize)]
struct UploadResp {
    url: String,
    delete_url: String,
}

#[derive(Serialize, Deserialize)]
struct Stats {
    files: usize,
    version: String,
}
// Use of a mod or pub mod is not actually necessary.
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn upload(
    mut parts: awmp::Parts,
    database: actix_web::web::Data<sled::Db>,
    settings: web::Data<ServerSettings>,
    request: HttpRequest,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let file_parts = parts
        .files
        .remove("file")
        .pop()
        .and_then(|f| f.persist("./uploads").ok())
        .unwrap_or_default();

    let (new_path, uri, ffn) = loop {
        let file_name = nanoid::generate(6);
        let folder_dir = nanoid::generate(2);

        let p = format!(
            "./uploads/{}/{}.{}",
            folder_dir,
            file_name,
            file_parts.extension().unwrap().to_str().unwrap()
        );

        if !std::path::Path::new(&p).exists() {
            if !std::path::Path::new(&format!("./uploads/{}", folder_dir)).exists() {
                fs::create_dir_all(format!("./uploads/{}", folder_dir)).unwrap();
            }
            break (
                p,
                format!("/{}/{}", folder_dir, file_name),
                format!(
                    "{}{}.{}",
                    folder_dir,
                    file_name,
                    file_parts.extension().unwrap().to_str().unwrap()
                ),
            );
        }
    };

    fs::rename(file_parts.display().to_string(), new_path.clone()).unwrap();

    let del_key = nanoid::simple();

    let ins =
        dbu::generate_insert_binary(new_path, del_key.clone(), request.connection_info().deref())
            .unwrap();

    database.insert(ffn.clone().into_bytes(), ins).unwrap();

    let resp_json = UploadResp {
        url: format!(
            "https://{}{}.{}",
            settings.website_name,
            uri,
            file_parts.extension().unwrap().to_str().unwrap()
        ),
        delete_url: format!(
            "https://{}/d{}.{}?del={}",
            settings.website_name,
            uri,
            file_parts.extension().unwrap().to_str().unwrap(),
            del_key
        ),
    };

    Ok(actix_web::HttpResponse::Ok().json(&resp_json))
}

fn serve(info: web::Path<FilePath>) -> actix_web::Result<NamedFile, actix_web::HttpResponse> {
    let file = format!("./uploads/{}/{}", info.folder, info.file);

    match NamedFile::open(file) {
        Ok(e) => Ok(e),
        Err(_e) => Err(actix_web::HttpResponse::NotFound().body(
            "the file you are looking for is either deleted or never existed in the first place",
        )),
    }
}

fn delete(
    path: web::Path<FilePath>,
    del: web::Query<DeleteFile>,
    database: web::Data<sled::Db>,
) -> HttpResponse {
    let binc = match database
        .get(format!("{}{}", path.folder, path.file))
        .unwrap()
    {
        Some(e) => e,
        None => {
            return HttpResponse::Unauthorized().body("this is not a valid file to delete");
        }
    };
    let data: dbu::FileMetadata = bincode::deserialize(&binc[..]).unwrap();

    if del.del != data.del_key {
        return HttpResponse::Unauthorized().body("invaild delete key");
    }

    database
        .remove(format!("{}{}", path.folder, path.file).into_bytes())
        .unwrap();

    let path = std::path::Path::new(&data.file_path);

    fs::remove_file(path).unwrap();

    if path.parent().unwrap().read_dir().unwrap().next().is_none() {
        fs::remove_dir(path.parent().unwrap()).unwrap();
    }

    HttpResponse::Ok().body("file deleted")
}

fn stats(database: web::Data<sled::Db>) -> HttpResponse {
    HttpResponse::Ok().json(Stats {
        files: database.len(),
        version: format!(
            "{} {}",
            built_info::PKG_VERSION,
            built_info::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!("(git {})", v))
        ),
    })
}

// TODO: Finish looking glass
/*fn looking_glass(
    databse: web::Data<sled::Db>,
    request: HttpRequest,
    settings: web::Data<ServerSettings>,
    info: web::Path<FilePath>,
) -> HttpResponse {
    let api_key = match request.headers().get("api") {
        None => return HttpResponse::Unauthorized().body("no api key supplied"),
        Some(e) => e,
    };

    if !settings
        .admin_keys
        .iter()
        .any(|x| x == api_key.to_str().unwrap())
    {
        return HttpResponse::Unauthorized().body("bad api key");
    }

    HttpResponse::Ok().body("coming soon")
}*/

fn p404() -> &'static str {
    "this resource does not exist."
}

fn main() {
    if !std::path::Path::new("./config.json").exists() {
        panic!("no config");
    }
    let config_json = fs::File::open("./config.json").unwrap();
    let server_settings: ServerSettings = serde_json::from_reader(config_json).unwrap();

    if !std::path::Path::new("./uploads").exists() {
        std::fs::create_dir_all("./uploads").unwrap();
    }

    let db = Db::open("db").unwrap();
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .data(db.clone())
            .data(server_settings.clone())
            .data(awmp::Parts::configure(|cfg| cfg.with_temp_dir("./uploads")))
            .route("/u", actix_web::web::post().to(upload))
            .route("/d/{folder}/{file}", web::get().to(delete))
            .route("/stats", web::get().to(stats))
            // .service(web::resource("/lg/{folder}/{file}").route(web::get().to(looking_glass)))
            .service(web::resource("/{folder}/{file}").route(web::get().to(serve)))
            .default_service(web::resource("").route(web::get().to(p404)))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .unwrap();
}

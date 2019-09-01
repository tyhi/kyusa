use crate::dbu;
use actix_web::{web, HttpResponse};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct FilePath {
    pub folder: String,
    pub file: String,
}
#[derive(Deserialize)]
pub struct DeleteFile {
    pub del: String,
}

pub fn delete(
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

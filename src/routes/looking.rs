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

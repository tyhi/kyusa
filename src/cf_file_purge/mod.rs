use crate::cf_file_purge::models::{PurgeFiles, Zones};
use isahc::prelude::*;

pub mod models;

pub fn get_domain_id(domain: &String, cf_api: &String) -> Option<String> {
    let resp = Request::get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", cf_api))
        .body(())
        .unwrap()
        .send()
        .unwrap()
        .json::<Zones>()
        .unwrap();

    for x in resp.result.iter() {
        if &x.name == domain {
            return Some(x.id.clone());
        }
    }
    None
}

pub fn purge_file(zone: &String, url: &String, key: &String) -> http::StatusCode {
    let files = PurgeFiles { files: vec![url] };
    let resp = Request::post(format!(
        "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
        zone
    ))
    .header("Authorization", format!("Bearer {}", key))
    .header("content-type", "application/json")
    .body(serde_json::to_vec(&files).unwrap())
    .unwrap()
    .send()
    .unwrap();

    resp.status()
}

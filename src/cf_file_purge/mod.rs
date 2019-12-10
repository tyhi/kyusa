use crate::cf_file_purge::models::{PurgeFiles, Zones};
use isahc::prelude::*;

pub mod models;

pub fn get_domain_id(
    domain: &String,
    cf_api: &String,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let resp = Request::get("https://api.cloudflare.com/client/v4/zones")
        .header("Authorization", format!("Bearer {}", cf_api))
        .body(())?
        .send()?
        .json::<Zones>()?;

    for x in resp.result.iter() {
        if &x.name == domain {
            return Ok(Some(x.id.clone()));
        }
    }
    Ok(None)
}

pub async fn purge_file(
    zone: &str,
    url: &String,
    key: &str,
) -> Result<isahc::http::StatusCode, Box<dyn std::error::Error>> {
    let files = PurgeFiles { files: vec![url] };
    let resp = Request::post(format!(
        "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
        zone
    ))
    .header("Authorization", format!("Bearer {}", key))
    .header("content-type", "application/json")
    .body(serde_json::to_vec(&files)?)?
    .send_async()
    .await?;

    Ok(resp.status())
}

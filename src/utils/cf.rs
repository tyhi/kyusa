use serde::Serialize;

#[derive(Serialize)]
pub struct PurgeFiles<'a> {
    pub files: Vec<&'a str>,
}

pub async fn purge(api_key: &str, zone_id: &str, url: &str) -> anyhow::Result<reqwest::StatusCode> {
    Ok(reqwest::Client::new()
        .post(&format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            zone_id
        ))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("content-type", "application/json")
        .body(serde_json::to_vec(&PurgeFiles { files: vec![url] })?)
        .send()
        .await?
        .status())
}

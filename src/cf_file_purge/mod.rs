use crate::cf_file_purge::models::Zones;
use isahc::prelude::*;

pub mod models;

pub fn get_domain_id(
    domain: &String,
    cf_api: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("deaths");
    let resp = Request::get("https://api.cloudflare.com/client/v4/zones")
        .header(
            "Authorization",
            "Bearer p6kAhtSnZo0vMasmnydv2Nz83AZVUcnt2YDJUfmo",
        )
        .body("")?
        .send()?
        .json::<Zones>()?;

    println!("{:?}", resp.result_info.count);
    Ok(String::new())
}

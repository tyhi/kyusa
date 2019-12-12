use addr::DomainName;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io, io::Read, path::Path};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub domain: String,
    pub domain_root: String,
    pub https: String,
    pub private: bool,
    pub port: String,
    pub multipart_name: String,
    pub key_details: HashMap<String, KeyDetails>,
    pub cloudflare_details: Option<CloudflareDetails>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudflareDetails {
    pub cf_zone: String,
    pub cf_api: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct KeyDetails {
    pub name: String,
    pub admin: bool,
}

impl Config {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        if !Path::new("./config.json").exists() {
            let config = init_cfg().await;

            let content = serde_json::to_string_pretty(&config)?;

            std::fs::write("./config.json", content)?;

            return Ok(config);
        }
        let mut file = File::open("./config.json")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let data: Config = serde_json::from_str(&contents).unwrap();
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn save(self, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(config)?;
        std::fs::write("./config.json", content)?;
        Ok(())
    }
}

async fn init_cfg() -> Config {
    println!("Enter port to be used to host web server:");
    let port = get_input();

    // Set domain used
    println!("Enter domain that will be used (with subdomain if used & no http(s)):");
    let domain_input = get_input();

    let domain: DomainName = domain_input.trim().parse().unwrap();
    let domain_root = domain.root().to_string();
    let domain_full = domain.to_string();

    // Check if will be using https
    let https: String;

    if parse_yn(
        "Is your domain setup to use https (eg. reverse proxy; this does not serve ssl)?[y/n]:",
    ) {
        https = "https".to_string()
    } else {
        https = "http".to_string()
    }
    let cf_details: Option<CloudflareDetails>;
    if parse_yn(
        "Do you want to setup CloudFlare integration? This will purge cache when you delete a \
         file.[y/n] ",
    ) {
        println!("Enter CloudFlare API Key (Permissions needed: Zone.Zone, Zone.Cache Purge):");
        let cf_api = get_input();
        let cf_zone = cfp_rs::get_domain_id(&domain_root, &cf_api)
            .await
            .expect("error getting domain id from cloudflare");
        cf_details = Some(CloudflareDetails { cf_zone, cf_api });
    } else {
        cf_details = None;
    }

    let mut api_keys: HashMap<String, KeyDetails> = HashMap::new();
    let private: bool;
    if parse_yn("Do you want to make this private? (e.g. require api keys?") {
        let api_key = nanoid::generate(24);
        println!(
            "This will be the admin api key it will only be shown once:\n{}",
            api_key
        );
        private = true;
        api_keys.insert(
            api_key,
            KeyDetails {
                name: "admin".to_string(),
                admin: true,
            },
        );
    } else {
        private = false;
    }

    // Setup public/private
    return Config {
        domain: domain_full,
        domain_root,
        port,
        https,
        multipart_name: "file".to_owned(),
        private,
        key_details: api_keys,
        cloudflare_details: cf_details,
    };
}

fn get_input() -> String {
    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("error reading stdin");
    s.trim().to_string()
}

fn parse_yn(question: &str) -> bool {
    loop {
        println!("{}", question);
        let s = get_input();
        if s == "y" || s == "yes" {
            return true;
        } else if s == "n" || s == "no" {
            return false;
        } else {
            println!("Not a valid entry. (valid: y/n/yes/no");
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Zones {
    pub result: Vec<Result>,
    pub result_info: ResultInfo,
    pub success: bool,
    pub errors: Vec<Option<serde_json::Value>>,
    pub messages: Vec<Option<serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
pub struct Result {
    pub id: String,
    pub name: String,
    pub status: String,
    pub paused: bool,
    #[serde(rename = "type")]
    pub result_type: String,
    pub development_mode: i64,
    pub name_servers: Vec<NameServer>,
    pub original_name_servers: Vec<String>,
    pub original_registrar: Option<String>,
    pub original_dnshost: Option<String>,
    pub modified_on: String,
    pub created_on: String,
    pub activated_on: String,
    pub meta: Meta,
    pub owner: Owner,
    pub account: Account,
    pub permissions: Vec<String>,
    pub plan: Plan,
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub step: i64,
    pub wildcard_proxiable: bool,
    pub custom_certificate_quota: i64,
    pub page_rule_quota: i64,
    pub phishing_detected: bool,
    pub multiple_railguns_allowed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Owner {
    pub id: String,
    #[serde(rename = "type")]
    pub owner_type: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub price: i64,
    pub currency: String,
    pub frequency: String,
    pub is_subscribed: bool,
    pub can_subscribe: bool,
    pub legacy_id: String,
    pub legacy_discount: bool,
    pub externally_managed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ResultInfo {
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
    pub count: i64,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize)]
pub enum NameServer {
    #[serde(rename = "janet.ns.cloudflare.com")]
    JanetNsCloudflareCom,
    #[serde(rename = "oswald.ns.cloudflare.com")]
    OswaldNsCloudflareCom,
}

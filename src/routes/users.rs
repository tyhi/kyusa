use actix_web::{get, post, web, Scope};

#[post("/user/register")]
pub async fn register() -> &'static str { "not impl" }

#[post("/user/delete")]
pub async fn del_user() -> &'static str { "not impl" }

#[get("/user/stats")]
pub async fn get_user_stats() -> &'static str { "not impl" }

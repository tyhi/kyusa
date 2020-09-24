use once_cell::sync::Lazy;
use short_url::UrlEncoder;
use std::env;

// "DEQhd2uFteibPwq0SWBInTpA_jcZL5GKz3YCR14Ulk87Jors9vNHgfaOmMXy6Vx"

pub static ENCODER: Lazy<UrlEncoder> = Lazy::new(|| {
    UrlEncoder::new(
        env::var("url_alph")
            .unwrap_or("DEQhd2uFteibPwq0SWBInTpA_jcZL5GKz3YCR14Ulk87Jors9vNHgfaOmMXy6Vx".into()),
        16,
    )
});

pub mod db;

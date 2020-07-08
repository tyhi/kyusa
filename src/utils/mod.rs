use once_cell::sync::Lazy;
use short_url::UrlEncoder;

pub static ENCODER: Lazy<UrlEncoder> = Lazy::new(|| {
    UrlEncoder::new(
        "DEQhd2uFteibPwq0SWBInTpA_jcZL5GKz3YCR14Ulk87Jors9vNHgfaOmMXy6Vx".into(),
        14,
    )
});

pub mod db;

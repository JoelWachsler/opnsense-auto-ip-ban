use std::env;

#[derive(Debug)]
pub struct Config {
    pub topic: String,
    pub group_id: String,
    pub bootstrap_servers: String,
    pub loki_url: String,
    pub opnsense_key: String,
    pub opnsense_secret: String,
    pub opnsense_alias_uuid: String,
    pub opnsense_host: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            topic: env::var("TOPIC").expect("TOPIC not set"),
            group_id: env::var("GROUP_ID").expect("GROUP_ID not set"),
            // group_id: "testing11".to_string(),
            bootstrap_servers: env::var("BOOTSTRAP_SERVERS").expect("BOOTSTRAP_SERVERS not set"),
            loki_url: env::var("LOKI_URL").expect("LOKI_URL not set"),
            opnsense_key: env::var("OPNSENSE_KEY").expect("OPNSENSE_KEY not set"),
            opnsense_secret: env::var("OPNSENSE_SECRET").expect("OPNSENSE_SECRET not set"),
            opnsense_alias_uuid: env::var("OPNSENSE_ALIAS_UUID")
                .expect("OPNSENSE_ALIAS_UUID not set"),
            opnsense_host: env::var("OPNSENSE_HOST")
                .unwrap_or_else(|_| "https://192.168.1.1".to_string()),
        }
    }
}

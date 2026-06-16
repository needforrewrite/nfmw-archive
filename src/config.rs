use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub asset_max_size_kb: u32,
    pub discord: DiscordConfig,
    pub bucket: BucketConfig
}

#[derive(Deserialize, Clone)]
pub struct DiscordConfig {
    pub client_id: i64,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Deserialize, Clone)]
pub struct BucketConfig {
    pub key_id: String,
    pub app_key: String,
    pub bucket_name: String
}

pub fn load_config() -> Config {
    let file = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str::<Config>(&file).unwrap()
}
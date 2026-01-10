use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub filestore: String,
}
    
pub fn load_config() -> Config {
    toml::from_str::<Config>(include_str!("../config.toml")).unwrap()
}
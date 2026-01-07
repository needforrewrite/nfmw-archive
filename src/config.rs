use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub filestore: String,
    pub database: DatabaseConfig,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub db_name: String,
}
impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.db_name
        )
    }
}
    
pub fn load_config() -> Config {
    toml::from_str::<Config>(include_str!("../config.toml")).unwrap()
}
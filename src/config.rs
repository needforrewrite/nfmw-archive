use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub asset_max_size_kb: u32,
    #[serde(default = "default_service_key_ttl_seconds")]
    pub service_key_ttl_seconds: i64,
    #[serde(default = "default_hmac_max_skew_seconds")]
    pub hmac_max_skew_seconds: i64,
    pub discord: DiscordConfig,
    pub bucket: BucketConfig,
    /// External services players can be issued scoped keys for, and which may
    /// authenticate to us with an HMAC credential. Empty is valid — it just
    /// means no service keys can be minted.
    #[serde(default)]
    pub services: Vec<ServiceConfig>
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

#[derive(Deserialize, Clone)]
pub struct ServiceConfig {
    /// The audience carried by service keys minted for this service, and the name
    /// it identifies itself by when signing requests to us. Clients ask for
    /// `?service=lobby`, never for a URL, so the URL below can change without
    /// anything outside this file knowing.
    pub id: String,
    /// Where a client should connect to reach this service. Handed back alongside
    /// the service key so the client never hardcodes it.
    pub url: String,
    /// Hex-encoded HMAC secrets this service may sign requests with. All are
    /// accepted, which is what makes rotation a config edit: add the new secret,
    /// move the service onto it, then drop the old one.
    pub hmac_secrets: Vec<String>
}

fn default_service_key_ttl_seconds() -> i64 {
    120
}

fn default_hmac_max_skew_seconds() -> i64 {
    30
}

impl Config {
    pub fn service(&self, id: &str) -> Option<&ServiceConfig> {
        self.services.iter().find(|s| s.id == id)
    }
}

pub fn load_config() -> Config {
    let file = std::fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str::<Config>(&file).unwrap();

    for service in &config.services {
        assert!(
            service.id != crate::database::account::ARCHIVE_AUDIENCE,
            "service id '{}' is reserved for this server's own sessions",
            crate::database::account::ARCHIVE_AUDIENCE
        );
        assert!(
            !service.hmac_secrets.is_empty(),
            "service '{}' has no hmac_secrets, so it could never authenticate",
            service.id
        );
        for secret in &service.hmac_secrets {
            let decoded = hex::decode(secret)
                .unwrap_or_else(|_| panic!("service '{}' has a non-hex hmac secret", service.id));
            assert!(
                decoded.len() >= 32,
                "service '{}' has an hmac secret shorter than 32 bytes",
                service.id
            );
        }
    }

    config
}

use cis_client::settings::CisSettings;
use config::{Config, ConfigError, Environment, File};
use dino_park_gate::settings::AuthValidationSettings;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct DinoParkSettings {
    pub search_update_endpoint: String,
    pub orgchart_update_endpoint: String,
    pub groups_update_endpoint: Option<String>,
    pub search_bulk_endpoint: String,
    pub orgchart_bulk_endpoint: String,
    pub groups_bulk_endpoint: Option<String>,
    pub search_delete_endpoint: String,
    pub orgchart_delete_endpoint: String,
    pub picture_delete_endpoint: String,
    pub groups_delete_endpoint: Option<String>,
    pub uuid_by_user_id_endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthSettings {
    pub issuer: String,
    pub validation: AuthValidationSettings,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub cis: CisSettings,
    pub dino_park: DinoParkSettings,
    pub auth: AuthSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let file = env::var("DPL_SETTINGS").unwrap_or_else(|_| String::from(".settings"));
        let mut s = Config::new();
        s.merge(File::with_name(&file))?;
        s.merge(Environment::new().separator("__").prefix("dp"))?;
        s.try_into()
    }
}

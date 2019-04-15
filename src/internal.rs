use crate::error::UpdateError;
use crate::settings::DinoParkSettings;
use cis_profile::schema::Profile;
use failure::Error;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

pub fn internal_update(dp: &DinoParkSettings, profile: &Profile) -> Result<Value, Error> {
    let id = profile
        .user_id
        .value
        .as_ref()
        .map(String::as_str)
        .unwrap_or_else(|| "unknown");
    info!("internally updating profile for: {}", id);
    Client::new()
        .post(&dp.orgchart_update_endpoint)
        .json(profile)
        .send()
        .map_err(UpdateError::OrgchartUpdate)?;
    info!("internally updated orgchart for: {}", id);
    Client::new()
        .post(&dp.search_update_endpoint)
        .json(profile)
        .send()
        .map_err(UpdateError::SearchUpdate)?;
    info!("internally updated search for: {}", id);
    Ok(json!({}))
}

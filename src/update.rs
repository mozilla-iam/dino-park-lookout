use crate::bulk::Bulk;
use crate::error::UpdateError;
use crate::notification::Notification;
use crate::settings::DinoParkSettings;
use cis_client::client::CisClientTrait;
use cis_client::client::GetBy;
use failure::Error;
use reqwest::multipart;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

pub fn update(
    cis_client: &impl CisClientTrait,
    dp: &DinoParkSettings,
    n: &Notification,
) -> Result<Value, Error> {
    info!("getting profile for: {}", &n.id);
    let profile = cis_client.get_user_by(&n.id, &GetBy::UserId, None)?;
    Client::new()
        .post(&dp.orgchart_update_endpoint)
        .json(&profile)
        .send()
        .map_err(UpdateError::OrgchartUpdate)?;
    info!("updated orgchart for: {}", &n.id);
    Client::new()
        .post(&dp.search_update_endpoint)
        .json(&profile)
        .send()
        .map_err(UpdateError::SearchUpdate)?;
    info!("updated search for: {}", &n.id);
    Ok(json!({}))
}

pub fn update_batch(
    cis_client: &impl CisClientTrait,
    dp: &DinoParkSettings,
    _: &Bulk,
) -> Result<Value, Error> {
    info!("getting bulk profiles");
    let profile_iter = cis_client.get_users_iter(None)?;
    for profiles in profile_iter {
        let profiles = profiles?;
        let mp = multipart::Part::text(serde_json::to_string(&profiles)?)
            .file_name("data")
            .mime_str("application/json")?;
        let form = multipart::Form::new().part("data", mp);
        Client::new()
            .post(&dp.orgchart_bulk_endpoint)
            .multipart(form)
            .send()
            .map_err(UpdateError::OrgchartUpdate)?;
        info!("updated orgchart for: {} profiles", profiles.len());
        let mp = multipart::Part::text(serde_json::to_string(&profiles)?)
            .file_name("data")
            .mime_str("application/json")?;
        let form = multipart::Form::new().part("data", mp);
        Client::new()
            .post(&dp.search_bulk_endpoint)
            .multipart(form)
            .send()
            .map_err(UpdateError::SearchUpdate)?;
        info!("updated search for: {} profiles", profiles.len());
    }
    Ok(json!({}))
}

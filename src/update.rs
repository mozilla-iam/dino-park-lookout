use crate::notification::Notification;
use crate::settings::DinoParkSettings;
use cis_client::client::CisClientTrait;
use cis_client::client::GetBy;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

pub fn update(
    cis_client: &impl CisClientTrait,
    dp: &DinoParkSettings,
    n: Notification,
) -> Result<Value, String> {
    info!("getting profile for: {}", &n.id);
    let profile = cis_client.get_user_by(&n.id, &GetBy::UserId, None)?;
    Client::new()
        .post(&dp.orgchart_update_endpoint)
        .json(&profile)
        .send()
        .map_err(|e| format!("error updating orgchart: {}", e))?;
    info!("updated orgchart for: {}", &n.id);
    Client::new()
        .post(&dp.search_update_endpoint)
        .json(&profile)
        .send()
        .map_err(|e| format!("error updating search: {}", e))?;
    info!("updated search for: {}", &n.id);
    Ok(json!({}))
}

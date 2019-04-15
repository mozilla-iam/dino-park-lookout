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
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::spawn;

#[derive(Clone)]
pub enum UpdateMessage {
    Notification(Notification),
    Bulk(Bulk),
    Stop,
}

pub trait UpdaterClient {
    fn update(&self, notification: Notification);
    fn update_all(&self, bulk: Bulk);
    fn stop(&self);
}

pub trait Updater<U: UpdaterClient> {
    fn client(&self) -> U;
}

#[derive(Clone)]
pub struct InternalUpdaterClient {
    sender: Sender<UpdateMessage>,
}

impl UpdaterClient for InternalUpdaterClient {
    fn update(&self, notification: Notification) {
        if let Err(e) = self.sender.send(UpdateMessage::Notification(notification)) {
            warn!("unable to send internally send notification: {}", e);
        }
    }
    fn update_all(&self, bulk: Bulk) {
        if let Err(e) = self.sender.send(UpdateMessage::Bulk(bulk)) {
            warn!("unable to send internally send notification: {}", e);
        }
    }
    fn stop(&self) {
        if let Err(e) = self.sender.send(UpdateMessage::Stop) {
            warn!("unable to send internally send stop message: {}", e);
        }
    }
}

pub struct InternalUpdater<T: CisClientTrait> {
    cis_client: T,
    dino_park_settings: DinoParkSettings,
    sender: Sender<UpdateMessage>,
    receiver: Receiver<UpdateMessage>,
}

impl<T: CisClientTrait + Clone + Sync + Send + 'static> InternalUpdater<T> {
    pub fn new(cis_client: T, dino_park_settings: DinoParkSettings) -> Self {
        let (sender, receiver) = channel();
        InternalUpdater {
            cis_client,
            dino_park_settings,
            sender,
            receiver,
        }
    }

    pub fn run(&self) -> Result<(), Error> {
        info!("start processing msgs");
        for msg in &self.receiver {
            if let UpdateMessage::Stop = msg {
                break;
            }
            let cis_client = self.cis_client.clone();
            let dino_park_settings = self.dino_park_settings.clone();
            spawn(move || {
                match msg {
                    UpdateMessage::Notification(n) => {
                        if let Err(e) = update(&cis_client, &dino_park_settings, &n) {
                            warn!("unable to update profile for {}: {}", &n.id, e);
                        };
                    }
                    UpdateMessage::Bulk(b) => {
                        if let Err(e) = update_batch(&cis_client, &dino_park_settings, &b) {
                            warn!("unable to bulk update profiles for: {}", e);
                        };
                    }
                    _ => {}
                };
            });
        }
        info!("stop processing msgs");
        Ok(())
    }
}

impl<T: CisClientTrait> Updater<InternalUpdaterClient> for InternalUpdater<T> {
    fn client(&self) -> InternalUpdaterClient {
        InternalUpdaterClient {
            sender: self.sender.clone(),
        }
    }
}

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

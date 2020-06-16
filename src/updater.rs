use crate::bulk::Bulk;
use crate::error::UpdateError;
use crate::notification::Notification;
use crate::notification::Operation;
use crate::settings::DinoParkSettings;
use cis_client::getby::GetBy;
use cis_client::sync::client::CisClientTrait;
use cis_client::AsyncCisClientTrait;
use cis_profile::schema::Profile;
use failure::Error;
use futures::future::join;
use futures::future::join3;
use futures::future::join4;
use futures::FutureExt;
use futures::TryFutureExt;
use reqwest::multipart;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::spawn;
use tokio::runtime::Runtime;

#[derive(Clone, Debug)]
pub enum UpdateMessage {
    Notification(Notification),
    Bulk(Bulk),
    Stop,
}

#[derive(Deserialize)]
struct UuidByUserId {
    uuid: Option<String>,
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

pub struct InternalUpdater<T: AsyncCisClientTrait + CisClientTrait> {
    cis_client: T,
    dino_park_settings: DinoParkSettings,
    sender: Sender<UpdateMessage>,
    receiver: Receiver<UpdateMessage>,
}

impl<T: AsyncCisClientTrait + CisClientTrait + Clone + Sync + Send + 'static> InternalUpdater<T> {
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
        let mut rt = Runtime::new()?;
        for msg in &self.receiver {
            debug!("got message: {:?}", msg);
            if let UpdateMessage::Stop = msg {
                break;
            }
            match msg {
                // Due to CIS sending Unknown instead of Delete we treat Unknown as Delete for now.
                UpdateMessage::Notification(n)
                    if n.operation == Operation::Delete || n.operation == Operation::Unknown =>
                {
                    let dino_park_settings = self.dino_park_settings.clone();
                    info!("processing");
                    if let Err(e) = rt.block_on(delete(&dino_park_settings, &n)) {
                        warn!("unable to delete profile for {}: {}", &n.id, e);
                    };
                }
                // UpdateMessage::Notification(n) if n.operation == Operation::Unknown => {
                //     warn!("received unknown operation");
                // }
                UpdateMessage::Notification(n) => {
                    let cis_client = self.cis_client.clone();
                    let dino_park_settings = self.dino_park_settings.clone();
                    info!("processing");
                    if let Err(e) = rt.block_on(update(&cis_client, &dino_park_settings, &n)) {
                        warn!("unable to update profile for {}: {}", &n.id, e);
                    };
                }
                UpdateMessage::Bulk(_) => {
                    let cis_client = self.cis_client.clone();
                    let dino_park_settings = self.dino_park_settings.clone();
                    spawn(move || {
                        debug!("processing");
                        if let Err(e) = update_batch(&cis_client, &dino_park_settings) {
                            warn!("unable to bulk update profiles for: {}", e);
                        };
                    });
                }
                _ => {}
            };
        }
        info!("stop processing msgs");
        Ok(())
    }
}

impl<T: AsyncCisClientTrait + CisClientTrait> Updater<InternalUpdaterClient>
    for InternalUpdater<T>
{
    fn client(&self) -> InternalUpdaterClient {
        InternalUpdaterClient {
            sender: self.sender.clone(),
        }
    }
}

pub async fn delete(dp: &DinoParkSettings, n: &Notification) -> Result<Value, Error> {
    let id = n.id.clone();
    let uuid = Client::new()
        .get(&format!("{}/{}", dp.uuid_by_user_id_endpoint, id))
        .send()
        .await?
        .json::<UuidByUserId>()
        .await?;
    if let Some(uuid) = uuid.uuid {
        let orgchart_delete = Client::new()
            .post(&format!("{}/{}", dp.orgchart_delete_endpoint, uuid))
            .send()
            .map_err(UpdateError::OrgchartDelete)
            .map_ok(|_| info!("deleted from orgchart: {}", &id));
        let search_delete = Client::new()
            .post(&format!("{}/{}", dp.search_delete_endpoint, uuid))
            .send()
            .map_err(UpdateError::SearchDelete)
            .map_ok(|_| info!("deleted from search: {}", &id));
        let picture_delete = Client::new()
            .post(&format!("{}/{}", dp.picture_delete_endpoint, uuid))
            .send()
            .map_err(UpdateError::PicturesDelete)
            .map_ok(|_| info!("deleted from pictures: {}", &id));
        if let Some(ref groups_delete_endpoint) = dp.groups_delete_endpoint {
            let groups_delete = Client::new()
                .delete(&format!("{}/{}", groups_delete_endpoint, uuid))
                .send()
                .map_err(UpdateError::GroupsDelete)
                .map_ok(|_| info!("updated groups for: {}", &id));
            join4(
                orgchart_delete,
                search_delete,
                picture_delete,
                groups_delete,
            )
            .map(|r| match r {
                (Ok(_), Ok(_), Ok(_), Ok(_)) => Ok(json!({})),
                _ => Err(UpdateError::Other.into()),
            })
            .await
        } else {
            join3(orgchart_delete, search_delete, picture_delete)
                .map(|r| match r {
                    (Ok(_), Ok(_), Ok(_)) => Ok(json!({})),
                    _ => Err(UpdateError::Other.into()),
                })
                .await
        }
    } else {
        error!("cannot resolve uuid for: {}", &id);
        Err(UpdateError::Other.into())
    }
}

pub async fn update(
    cis_client: &impl AsyncCisClientTrait,
    dp: &DinoParkSettings,
    n: &Notification,
) -> Result<Value, Error> {
    info!("getting profile for: {}", &n.id);
    let profile = match cis_client.get_user_by(&n.id, &GetBy::UserId, None).await {
        Ok(p) => p,
        Err(_) => {
            cis_client
                .get_inactive_user_by(&n.id, &GetBy::UserId, None)
                .await?
        }
    };
    info!(
        "{} is active: {}",
        profile.user_id.value.as_deref().unwrap_or_else(|| "?"),
        profile.active.value.as_ref().unwrap_or_else(|| &false)
    );
    send_profile(dp, profile).await
}

pub async fn send_profile(dp: &DinoParkSettings, profile: Profile) -> Result<Value, Error> {
    let id = profile
        .user_id
        .value
        .clone()
        .unwrap_or_else(|| String::from("unknown"));
    let orgchart_update = Client::new()
        .post(&dp.orgchart_update_endpoint)
        .json(&profile)
        .send()
        .map_err(UpdateError::OrgchartUpdate)
        .map_ok(|_| info!("updated orgchart for: {}", &id));
    let search_update = Client::new()
        .post(&dp.search_update_endpoint)
        .json(&profile)
        .send()
        .map_err(UpdateError::SearchUpdate)
        .map_ok(|_| info!("updated search for: {}", &id));
    if let Some(ref groups_update_endpoint) = dp.groups_update_endpoint {
        let groups_update = Client::new()
            .post(groups_update_endpoint)
            .json(&profile)
            .send()
            .map_err(UpdateError::GroupsUpdate)
            .map_ok(|_| info!("updated groups for: {}", &id));
        join3(orgchart_update, search_update, groups_update)
            .map(|r| match r {
                (Ok(_), Ok(_), Ok(_)) => Ok(json!({})),
                _ => Err(UpdateError::Other.into()),
            })
            .await
    } else {
        join(orgchart_update, search_update)
            .map(|r| match r {
                (Ok(_), Ok(_)) => Ok(json!({})),
                _ => Err(UpdateError::Other.into()),
            })
            .await
    }
}

pub fn update_batch(
    cis_client: &impl CisClientTrait,
    dp: &DinoParkSettings,
) -> Result<Value, Error> {
    debug!("getting bulk profiles");
    let mut rt = Runtime::new()?;
    let profiles_iter = cis_client.get_users_iter(None)?;
    for profiles in profiles_iter {
        if let Ok(profiles) = profiles {
            info!("{}", profiles.len());
            rt.block_on(
                async move {
                    let mp = multipart::Part::text(serde_json::to_string(&profiles)?)
                        .file_name("data")
                        .mime_str("application/json")?;
                    let form = multipart::Form::new().part("data", mp);
                    let orgchart_update = Client::new()
                        .post(&dp.orgchart_bulk_endpoint)
                        .multipart(form)
                        .send()
                        .map_err(UpdateError::OrgchartUpdate)
                        .map_err(|e| {
                            error!("batch: {}", e);
                            e
                        })
                        .map_ok(|_| info!("updated orgchart for: {} profiles", profiles.len()));
                    let mp = multipart::Part::text(serde_json::to_string(&profiles)?)
                        .file_name("data")
                        .mime_str("application/json")?;
                    let form = multipart::Form::new().part("data", mp);
                    let search_update = Client::new()
                        .post(&dp.search_bulk_endpoint)
                        .multipart(form)
                        .send()
                        .map_err(UpdateError::SearchUpdate)
                        .map_err(|e| {
                            error!("batch: {}", e);
                            e
                        })
                        .map_ok(|_| info!("updated search for: {} profiles", profiles.len()));
                    if let Some(ref groups_bulk_endpoint) = dp.groups_bulk_endpoint {
                        let mp = multipart::Part::text(serde_json::to_string(&profiles)?)
                            .file_name("data")
                            .mime_str("application/json")?;
                        let form = multipart::Form::new().part("data", mp);
                        let groups_update = Client::new()
                            .post(groups_bulk_endpoint)
                            .multipart(form)
                            .send()
                            .map_err(UpdateError::GroupsUpdate)
                            .map_err(|e| {
                                error!("batch: {}", e);
                                e
                            })
                            .map_ok(|_| info!("updated groups for: {} profiles", profiles.len()));
                        join3(orgchart_update, search_update, groups_update)
                            .map(|_| Ok::<(), Error>(()))
                            .await
                    } else {
                        join(orgchart_update, search_update)
                            .map(|_| Ok::<(), Error>(()))
                            .await
                    }
                }
                .map(|r| {
                    if let Err(e) = r {
                        error!("unable to process batch: {}", e)
                    };
                }),
            )
        }
    }
    info!("done bulk updating");
    Ok(json!({}))
}

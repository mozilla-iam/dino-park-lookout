use crate::bulk::Bulk;
use crate::notification::Notification;
use crate::settings::DinoParkSettings;
use crate::update::update;
use crate::update::update_batch;
use cis_client::client::CisClientTrait;
use failure::Error;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::spawn;

#[derive(Clone)]
pub enum UpdateMessage {
    Notification(Notification),
    Bulk(Bulk),
}

pub trait UpdaterClient {
    fn update(&mut self, notification: Notification);
    fn update_all(&mut self, bulk: Bulk);
}

pub trait Updater<U: UpdaterClient> {
    fn client(&self) -> U;
}

#[derive(Clone)]
pub struct InternalUpdaterClient {
    sender: Sender<UpdateMessage>,
}

impl UpdaterClient for InternalUpdaterClient {
    fn update(&mut self, notification: Notification) {
        if let Err(e) = self.sender.send(UpdateMessage::Notification(notification)) {
            warn!("unable to send internally send notification: {}", e);
        }
    }
    fn update_all(&mut self, bulk: Bulk) {
        if let Err(e) = self.sender.send(UpdateMessage::Bulk(bulk)) {
            warn!("unable to send internally send notification: {}", e);
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
        for msg in &self.receiver {
            let cis_client = self.cis_client.clone();
            let dino_park_settings = self.dino_park_settings.clone();
            spawn(move || match msg {
                UpdateMessage::Notification(n) => update(&cis_client, &dino_park_settings, &n),
                UpdateMessage::Bulk(b) => update_batch(&cis_client, &dino_park_settings, &b),
            });
        }
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

use crate::settings::DinoParkSettings;
use cis_client::client::CisClientTrait;
use std::sync::Arc;

pub struct AppState<T: CisClientTrait + Clone + 'static> {
    pub cis_client: T,
    pub dino_park_settings: Arc<DinoParkSettings>,
}

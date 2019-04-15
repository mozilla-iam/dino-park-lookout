use crate::settings::DinoParkSettings;
use crate::updater::UpdaterClient;
use std::sync::Arc;

pub struct AppState<U: UpdaterClient + Clone + 'static> {
    pub dino_park_settings: Arc<DinoParkSettings>,
    pub updater: U,
}

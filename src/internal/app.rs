use crate::bulk::Bulk;
use crate::settings::DinoParkSettings;
use crate::updater::send_profile;
use crate::updater::UpdaterClient;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::error::Error;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use actix_web::Result;
use cis_profile::schema::Profile;
use futures::future::TryFutureExt;
use serde_json::json;
use serde_json::Value;

pub async fn internal_update(dp: &DinoParkSettings, profile: Profile) -> Result<Value, Error> {
    send_profile(dp, profile).map_err(Into::into).await
}

async fn internal_update_event(
    dino_park_settings: Data<DinoParkSettings>,
    profile: Json<Profile>,
) -> Result<HttpResponse, Error> {
    let id = profile
        .user_id
        .value
        .clone()
        .unwrap_or_else(|| String::from("unknown"));
    info!("internally updating profile for: {}", &id);
    let id_c = id.clone();
    let res = internal_update(&dino_park_settings, profile.into_inner()).await;
    info!("internally updated profile for {}", id);
    match res {
        Ok(j) => Ok(HttpResponse::Ok().json(j)),
        Err(e) => {
            error!("failed to internally update profile for {}: {}", id_c, e);
            Err(error::ErrorInternalServerError(e))
        }
    }
}

async fn bulk_update<U: UpdaterClient + Clone + 'static>(
    updater: Data<U>,
    bulk: Json<Bulk>,
) -> Result<HttpResponse> {
    updater.update_all(bulk.0);
    Ok(HttpResponse::Ok().json(json!({})))
}

pub fn internal_app<U: UpdaterClient + Clone + Send + 'static>(
    dino_park_settings: DinoParkSettings,
    updater: U,
) -> impl HttpServiceFactory {
    web::scope("")
        .data(updater)
        .data(dino_park_settings)
        .app_data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/bulk").route(web::post().to(bulk_update::<U>)))
        .service(web::resource("/update").route(web::post().to(internal_update_event)))
}

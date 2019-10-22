use crate::bulk::Bulk;
use crate::settings::DinoParkSettings;
use crate::updater::UpdaterClient;
use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::dev::HttpServiceFactory;
use actix_web::error;
use actix_web::error::Error;
use actix_web::http;
use actix_web::web;
use actix_web::web::Data;
use actix_web::web::Json;
use actix_web::HttpResponse;
use actix_web::Result;
use cis_profile::schema::Profile;
use futures::Future;
use serde_json::json;
use serde_json::Value;

pub fn internal_update(
    dp: &DinoParkSettings,
    profile: &Profile,
) -> impl Future<Item = Value, Error = Error> {
    let id = profile
        .user_id
        .value
        .clone()
        .unwrap_or_else(|| String::from("unknown"));
    info!("internally updating profile for: {}", &id);
    let id_c = id.clone();
    let orgchart_update = Client::default()
        .post(&dp.orgchart_update_endpoint)
        .send_json(profile)
        .map(move |_| info!("internally updated orgchart for: {}", id));

    orgchart_update
        .join(
            Client::default()
                .post(&dp.search_update_endpoint)
                .send_json(profile)
                .map(move |_| info!("internally updated search for: {}", id_c)),
        )
        .map(|_| json!({}))
        .map_err(Into::into)
}

fn internal_update_event(
    dino_park_settings: Data<DinoParkSettings>,
    profile: Json<Profile>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = profile
        .user_id
        .value
        .clone()
        .unwrap_or_else(|| String::from("unknown"));
    info!("internally updating profile for: {}", &id);
    let id_c = id.clone();
    internal_update(&dino_park_settings, &profile)
        .map(move |res| {
            info!("internally updated profile for {}", id);
            HttpResponse::Ok().json(res)
        })
        .map_err(move |e| {
            error!("failed to internally update profile for {}: {}", id_c, e);
            error::ErrorInternalServerError(e)
        })
        .map_err(Into::into)
}

fn bulk_update<U: UpdaterClient + Clone + 'static>(
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
        .wrap(
            Cors::new()
                .allowed_methods(vec!["POST"])
                .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                .allowed_header(http::header::CONTENT_TYPE)
                .max_age(3600),
        )
        .data(updater)
        .data(dino_park_settings)
        .data(web::JsonConfig::default().limit(1_048_576))
        .service(web::resource("/bulk").route(web::post().to(bulk_update::<U>)))
        .service(web::resource("/update").route(web::post().to_async(internal_update_event)))
}

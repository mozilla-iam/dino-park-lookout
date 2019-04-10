use crate::bulk::Bulk;
use crate::notification::Notification;
use crate::settings::DinoParkSettings;
use crate::state::AppState;
use crate::update::internal_update;
use crate::update::update;
use crate::update::update_batch;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::Json;
use actix_web::Result;
use actix_web::State;
use cis_client::client::CisClientTrait;
use cis_profile::schema::Profile;
use dino_park_gate::check::TokenChecker;
use dino_park_gate::middleware::AuthMiddleware;
use std::sync::Arc;

fn update_event<T: CisClientTrait + Clone + 'static>(
    state: State<AppState<T>>,
    n: Json<Notification>,
) -> Result<HttpResponse> {
    match update(&state.cis_client, &state.dino_park_settings, &n) {
        Ok(res) => {
            info!("updated profile for {}", &n.id);
            let res_text = serde_json::to_string(&res)?;
            Ok(HttpResponse::Ok().json(res_text))
        }
        Err(e) => {
            error!("failed to update profile for {}: {}", &n.id, e);
            Err(error::ErrorInternalServerError(e))
        }
    }
}

fn internal_update_event<T: CisClientTrait + Clone + 'static>(
    state: State<AppState<T>>,
    profile: Json<Profile>,
) -> Result<HttpResponse> {
    let id = profile
        .user_id
        .value
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or_else(|| "unknown");
    match internal_update(&state.dino_park_settings, &profile) {
        Ok(res) => {
            info!("internally updated profile for {}", id);
            let res_text = serde_json::to_string(&res)?;
            Ok(HttpResponse::Ok().json(res_text))
        }
        Err(e) => {
            error!("failed to internally update profile for {}: {}", id, e);
            Err(error::ErrorInternalServerError(e))
        }
    }
}

fn bulk_update<T: CisClientTrait + Clone + 'static>(
    state: State<AppState<T>>,
    bulk: Json<Bulk>,
) -> Result<HttpResponse> {
    match update_batch(&state.cis_client, &state.dino_park_settings, &bulk) {
        Ok(res) => {
            info!("bulk updated profiles");
            let res_text = serde_json::to_string(&res)?;
            Ok(HttpResponse::Ok().json(res_text))
        }
        Err(e) => {
            error!("failed to bulk update profiles: {}", e);
            Err(error::ErrorInternalServerError(e))
        }
    }
}

pub fn app<T: CisClientTrait + Clone + Send + Sync + 'static>(
    cis_client: T,
    dino_park_settings: DinoParkSettings,
    auth_middleware: AuthMiddleware<impl TokenChecker + Clone + 'static>,
) -> App<AppState<T>> {
    let dino_park_settings_arc = Arc::new(dino_park_settings);
    let state = AppState {
        cis_client,
        dino_park_settings: dino_park_settings_arc,
    };
    App::with_state(state).configure(|app| {
        let f = auth_middleware.clone();
        Cors::for_app(app)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/internal/update", |r| {
                r.method(http::Method::POST).with(internal_update_event)
            })
            .resource("/events/update", move |r| {
                r.middleware(f);
                r.method(http::Method::POST).with(update_event)
            })
            .resource("/bulk/update", |r| {
                r.method(http::Method::POST).with(bulk_update)
            })
            .register()
    })
}

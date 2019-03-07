use crate::notification::Notification;
use crate::settings::DinoParkSettings;
use crate::update::update;
use actix::prelude::*;
use actix_web::error;
use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::App;
use actix_web::AsyncResponder;
use actix_web::Error;
use actix_web::FutureResponse;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use cis_client::client::CisClientTrait;
use futures::Future;
use std::sync::Arc;

struct UpdateExecutor<T: CisClientTrait + Clone> {
    cis_client: T,
    dino_park_settings: Arc<DinoParkSettings>,
}

pub struct AppState<T: CisClientTrait + Clone + 'static> {
    executor: Addr<UpdateExecutor<T>>,
}

fn update_event<T: CisClientTrait + Clone + 'static>(
    req: HttpRequest<AppState<T>>,
) -> FutureResponse<HttpResponse> {
    req.json::<Notification>()
        .from_err()
        .and_then(move |n| {
            req.state()
                .executor
                .send(n)
                .from_err()
                .and_then(|res| match res {
                    Ok(v) => Ok(HttpResponse::Ok().content_type("application/json").body(v)),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

impl<T: CisClientTrait + Clone + 'static> UpdateExecutor<T> {
    pub fn new(cis_client: T, dino_park_settings: Arc<DinoParkSettings>) -> Self {
        UpdateExecutor {
            cis_client,
            dino_park_settings,
        }
    }
}

impl<T: CisClientTrait + Clone + 'static> Actor for UpdateExecutor<T> {
    type Context = SyncContext<Self>;
}

impl<T: CisClientTrait + Clone + 'static> Handler<Notification> for UpdateExecutor<T> {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: Notification, _: &mut Self::Context) -> Self::Result {
        let res = update(&self.cis_client, &self.dino_park_settings, msg)
            .map_err(error::ErrorRequestTimeout)?;
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

pub fn update_app<T: CisClientTrait + Clone + Send + Sync + 'static>(
    cis_client: T,
    dino_park_settings: DinoParkSettings,
) -> App<AppState<T>> {
    let dino_park_settings_arc = Arc::new(dino_park_settings);
    let addr = SyncArbiter::start(3, move || {
        UpdateExecutor::new(cis_client.clone(), dino_park_settings_arc.clone())
    });

    App::with_state(AppState {
        executor: addr.clone(),
    })
    .configure(|app| {
        Cors::for_app(app)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600)
            .resource("/events/update", |r| {
                r.method(http::Method::POST).with(update_event)
            })
            .register()
    })
}

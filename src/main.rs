extern crate actix;
extern crate actix_web;
extern crate biscuit;
extern crate chrono;
extern crate cis_client;
extern crate cis_profile;
extern crate condvar_store;
extern crate config;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate url;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod app;
mod auth;
mod bulk;
mod error;
mod notification;
mod settings;
mod state;
mod update;

use crate::app::app;
use crate::auth::middleware::AuthMiddleware;
use crate::auth::provider::Provider;
use actix_web::middleware;
use actix_web::server;
use cis_client::client::CisClient;

fn main() -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_lookout=info");
    env_logger::init();
    info!("building the lookout");
    let sys = actix::System::new("dino-park-lookout");
    let s = settings::Settings::new().map_err(|e| format!("unable to load settings: {}", e))?;
    let cis_client = CisClient::from_settings(&s.cis)
        .map_err(|e| format!("unable to create cis_client: {}", e))?;
    let dino_park = s.dino_park.clone();
    let validation_settings = s.auth.validation.clone();
    let provider = Provider::from_issuer(&s.auth.issuer).map_err(|e| e.to_string())?;
    // Start http server
    let auth_middleware = AuthMiddleware {
        checker: provider,
        validation_options: validation_settings.to_validation_options(),
    };
    server::new(move || {
        vec![app(
            cis_client.clone(),
            dino_park.clone(),
            auth_middleware.clone(),
        )
        .middleware(middleware::Logger::default())
        .boxed()]
    })
    .bind("0.0.0.0:8082")
    .unwrap()
    .start();

    info!("Started http server");
    let _ = sys.run();
    Ok(())
}

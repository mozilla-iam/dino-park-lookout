extern crate actix;
extern crate actix_web;
extern crate biscuit;
extern crate chrono;
extern crate cis_client;
extern crate cis_profile;
extern crate condvar_store;
extern crate config;
extern crate dino_park_gate;
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
mod bulk;
mod error;
mod internal;
mod notification;
mod settings;
mod state;
mod updater;

use crate::app::app;
use crate::updater::InternalUpdater;
use crate::updater::Updater;
use crate::updater::UpdaterClient;
use actix_web::middleware;
use actix_web::server;
use cis_client::client::CisClient;
use dino_park_gate::middleware::AuthMiddleware;
use dino_park_gate::provider::Provider;
use std::thread::spawn;

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

    let updater = InternalUpdater::new(cis_client.clone(), dino_park.clone());

    let client = updater.client();
    let stop_client = updater.client();
    let updater_thread = spawn(move || {
        if let Err(e) = updater.run() {
            error!("unable to start updater: {}", e);
        }
    });
    server::new(move || {
        vec![
            app(dino_park.clone(), client.clone(), auth_middleware.clone())
                .middleware(middleware::Logger::default())
                .boxed(),
        ]
    })
    .bind("0.0.0.0:8082")
    .unwrap()
    .start();

    info!("Started http server");
    let _ = sys.run();
    info!("Stopped http server");
    stop_client.stop();
    updater_thread
        .join()
        .map_err(|_| String::from("failed to stop updater"))?;
    Ok(())
}

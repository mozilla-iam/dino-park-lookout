#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod bulk;
mod error;
mod events;
mod healthz;
mod internal;
mod notification;
mod settings;
mod updater;

use crate::events::app::update_app;
use crate::healthz::healthz_app;
use crate::internal::app::internal_app;
use crate::updater::InternalUpdater;
use crate::updater::Updater;
use crate::updater::UpdaterClient;
use actix_rt::System;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use cis_client::CisClient;
use dino_park_gate::provider::Provider;
use dino_park_gate::simple::SimpleAuth;
use failure::format_err;
use failure::Error;
use std::thread::spawn;

fn main() -> Result<(), Error> {
    ::std::env::set_var(
        "RUST_LOG",
        "actix_web=info,dino_park_lookout=info,dino_park_gate=info,cis_client=info,shared_expiry_get=info",
    );
    env_logger::init();
    info!("building the lookout");
    let s = settings::Settings::new().map_err(|e| format_err!("unable to load settings: {}", e))?;
    let cis_client = CisClient::from_settings(&s.cis)
        .map_err(|e| format_err!("unable to create cis_client: {}", e))?;
    let dino_park = s.dino_park.clone();
    let validation_settings = s.auth.validation.clone();
    let mut rt = tokio::runtime::Runtime::new()?;
    let provider = rt.block_on(Provider::from_issuer(&s.auth.issuer))?;
    // Start http server
    let updater = InternalUpdater::new(cis_client, dino_park.clone());

    let client = updater.client();
    let stop_client = updater.client();
    let updater_thread = spawn(move || {
        if let Err(e) = updater.run() {
            error!("unable to start updater: {}", e);
        }
    });
    let server = HttpServer::new(move || {
        let auth_middleware = SimpleAuth {
            checker: provider.clone(),
            validation_options: validation_settings.to_validation_options(),
        };

        App::new()
            .wrap(Logger::default().exclude("/healthz"))
            .service(
                web::scope("/internal").service(internal_app(dino_park.clone(), client.clone())),
            )
            .service(
                web::scope("/events")
                    .wrap(auth_middleware)
                    .service(update_app(client.clone())),
            )
            .service(healthz_app())
    })
    .bind("0.0.0.0:8082")?;

    System::new("lookout-actix-rt").block_on(async move { server.run().await })?;

    info!("Stopped http server");
    stop_client.stop();
    updater_thread
        .join()
        .map_err(|_| format_err!("failed to stop updater"))?;
    Ok(())
}

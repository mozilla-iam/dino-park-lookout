extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate cis_client;
extern crate cis_profile;
extern crate config;
extern crate env_logger;
extern crate failure;
extern crate futures;
extern crate reqwest;
extern crate serde;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod bulk;
mod error;
mod handler;
mod notification;
mod settings;
mod update;

use actix_web::middleware;
use actix_web::server;
use cis_client::client::CisClient;
use handler::update_app;

fn main() -> Result<(), String> {
    ::std::env::set_var("RUST_LOG", "actix_web=info,dino_park_lookout=info");
    env_logger::init();
    info!("building the lookout");
    let sys = actix::System::new("dino-park-lookout");
    let s = settings::Settings::new().map_err(|e| format!("unable to load settings: {}", e))?;
    let cis_client = CisClient::from_settings(&s.cis)
        .map_err(|e| format!("unable to create cis_client: {}", e))?;
    let dino_park = s.dino_park.clone();
    // Start http server
    server::new(move || {
        vec![update_app(cis_client.clone(), dino_park.clone())
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

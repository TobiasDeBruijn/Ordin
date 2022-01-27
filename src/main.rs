use crate::appdata::ApplicationData;
use crate::config::Config;
use crate::opts::Opts;
use crate::services::ansible::AnsibleService;
use crate::services::dns::DnsService;
use actix_web::middleware::normalize::TrailingSlash;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use log::{error, info, LevelFilter};
use std::process::exit;

mod appdata;
mod config;
mod error;
mod handlers;
mod opts;
mod services;
mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts = Opts::new();
    match opts.verbose {
        0 => env_logger::builder()
            .filter_level(LevelFilter::Error)
            .init(),
        1 => env_logger::builder().filter_level(LevelFilter::Warn).init(),
        2 => env_logger::builder().filter_level(LevelFilter::Info).init(),
        3 => env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .init(),
        _ => env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .init(),
    }

    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    let config = match Config::from_file(&opts.config) {
        Ok(x) => x,
        Err(e) => {
            error!("Failed to load config: {:?}", e);
            exit(1);
        }
    };

    let ansible_service = AnsibleService::new(&config).expect("Creating Ansible service");
    let dns_service = DnsService::new(&config);
    let appdata = ApplicationData::new(config, ansible_service, dns_service);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .data(appdata.clone())
            .route(
                "phone-home",
                web::post().to(handlers::phone_home::phone_home),
            )
    })
    .bind("[::]:4040")?
    .run()
    .await
}

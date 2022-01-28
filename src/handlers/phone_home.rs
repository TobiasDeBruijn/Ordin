use crate::appdata::WebData;
use crate::error::ServiceResult;
use crate::handlers::{Empty, Sender};
use crate::services::{Service, Target};
use actix_web::web;
use serde::Deserialize;
use std::thread;

#[derive(Deserialize)]
pub struct Request {
    hostname: String,
}

pub async fn phone_home(
    data: WebData,
    payload: web::Form<Request>,
    sender: Sender,
) -> ServiceResult<Empty> {
    thread::Builder::new()
        .name(format!("phone-home-{}-{}", &payload.hostname, &sender.ip))
        .spawn(move || {
            let target = Target::new(&sender.ip, &payload.hostname);
            let dns = data.dns.clone();
            let ansible = data.ansible.clone();

            dns.run(&target).expect("Running DNS service");
            ansible.run(&target).expect("Running Ansible service");
        })
        .expect("Spawning thread");

    Ok(Empty)
}

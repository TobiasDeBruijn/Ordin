use crate::config::Config;
use crate::services::ansible::AnsibleService;
use crate::DnsService;
use actix_web::web;
use std::sync::Arc;

pub type WebData = web::Data<Arc<ApplicationData>>;

#[derive(Debug)]
pub struct ApplicationData {
    pub config: Config,
    pub ansible: AnsibleService,
    pub dns: DnsService,
}

impl ApplicationData {
    pub fn new(config: Config, ansible: AnsibleService, dns: DnsService) -> Arc<Self> {
        Arc::new(Self {
            config,
            ansible,
            dns,
        })
    }
}

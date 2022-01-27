use actix_web::http::StatusCode;
use actix_web::ResponseError;
use thiserror::Error;

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("DNS error")]
    Dns(#[from] crate::services::dns::DnsError),
    #[error("Ansible error")]
    Ansible(#[from] crate::services::ansible::AnsibleError),
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Dns(_) | Self::Ansible(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

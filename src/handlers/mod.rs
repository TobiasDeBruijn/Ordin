use crate::util::Ready;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest, HttpResponse, Responder};

pub mod phone_home;

/// Empty response
pub struct Empty;

impl Responder for Empty {
    type Error = ();
    type Future = Ready<Result<HttpResponse, ()>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        Ready::new(Ok(HttpResponse::Ok().finish()))
    }
}

/// The sender of the request
pub struct Sender {
    /// The real IP address of the sender
    pub ip: String,
}

impl FromRequest for Sender {
    type Error = ();
    type Future = Ready<Result<Self, ()>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let conn_info = req.connection_info();
        let ip = match conn_info.realip_remote_addr() {
            Some(x) => x.to_string(),
            None => return Ready::new(Err(())),
        };

        let ip = ip.replace('[', "").replace(']', "");
        let mut ip_parts = ip.split(':').collect::<Vec<_>>();
        ip_parts.pop();
        let ip = ip_parts.join(":");
        Ready::new(Ok(Self { ip }))
    }
}

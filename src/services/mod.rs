pub mod ansible;
pub mod dns;

#[derive(Debug)]
pub struct Target {
    pub ip: String,
    pub hostname: String,
}

impl Target {
    pub fn new<S, S1>(ip: S, hostname: S1) -> Self
    where
        S: AsRef<str>,
        S1: AsRef<str>,
    {
        Self {
            ip: ip.as_ref().to_string(),
            hostname: hostname.as_ref().to_string(),
        }
    }
}

pub trait Service {
    type Err;
    fn run(&self, target: &Target) -> Result<(), Self::Err>;
}

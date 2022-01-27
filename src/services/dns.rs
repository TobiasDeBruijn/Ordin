use crate::services::{Service, Target};
use crate::Config;
use log::trace;
use std::io::Write;
use std::net::Ipv6Addr;
use std::process::{Command, Stdio};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("IO Error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Nsupdate failed")]
    NsupdateFailed,
    #[error("Failed to parse network address {0:?}")]
    AddrParse(#[from] std::net::AddrParseError),
}

#[derive(Debug, Clone)]
pub struct DnsService {
    server: String,
    zone: String,
    ttl: u64,
    domain: String,
}

impl Service for DnsService {
    type Err = DnsError;
    fn run(&self, target: &Target) -> Result<(), Self::Err> {
        self.add_record(target)
    }
}

impl DnsService {
    pub fn new(config: &Config) -> Self {
        Self {
            server: config.dns.server.clone(),
            zone: config.dns.zone_name.clone(),
            ttl: config.dns.ttl,
            domain: config.global.domain.clone(),
        }
    }

    fn fmt_hostname(&self, target: &Target) -> String {
        format!("{}.{}", target.hostname, self.domain)
    }

    fn add_record(&self, target: &Target) -> Result<(), DnsError> {
        let mut child = Command::new("nsupdate")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.as_mut().unwrap();
        trace!("Nsupdate: server {}", &self.server);
        stdin.write_all(format!("server {}\n", &self.server).as_bytes())?;
        trace!("Nsupdate: zone {}", &self.zone);
        stdin.write_all(format!("zone {}\n", &self.zone).as_bytes())?;

        let target_hostname = self.fmt_hostname(target);
        let record = if target_hostname.ends_with(&self.zone) {
            target_hostname
        } else {
            format!("{}.{}", target_hostname, self.zone)
        };

        if target.ip.contains(':') {
            let ip = Ipv6Addr::from_str(&target.ip)?;
            let ip = ip
                .segments()
                .as_slice()
                .iter()
                .map(|x| format!("{:0>4}", x))
                .collect::<Vec<_>>()
                .join(":");
            trace!("Nsupdate: update add {} {} AAAA {}", record, self.ttl, ip);
            stdin.write_all(
                format!("update add {} {} AAAA {}\n", record, self.ttl, ip).as_bytes(),
            )?;
        } else {
            trace!(
                "Nsupdate: update add {} {} A {}",
                record,
                self.ttl,
                target.ip
            );
            stdin.write_all(
                format!("update add {} {} A {}\n", record, self.ttl, target.ip).as_bytes(),
            )?;
        }

        trace!("Nsupdate: send");
        stdin.write_all(b"send\n")?;
        trace!("Nsupdate: quit");
        stdin.write_all(b"quit\n")?;

        let output = child.wait_with_output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        trace!("Nsupdate stdout: {:?}", stdout);
        trace!("Nsupdate stderr: {:?}", stderr);

        if !output.status.success() {
            return Err(DnsError::NsupdateFailed);
        }

        trace!("Nsupdate completed successfully");
        Ok(())
    }
}

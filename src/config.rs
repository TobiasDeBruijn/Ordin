use log::{debug, trace, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Config {
    pub ansible: AnsibleConfig,
    pub dns: DnsConfig,
    pub global: GlobalConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GlobalConfig {
    pub domain: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct DnsConfig {
    pub server: String,
    pub zone_name: String,
    pub ttl: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AnsibleConfig {
    pub ansible_playbook_binary: Option<PathBuf>,
    pub playbooks: Vec<PathBuf>,
    pub inventory: PathBuf,
    pub play_logs: bool,
    #[serde(default = "default_play_logdir")]
    pub play_logdir: PathBuf,
}

fn default_play_logdir() -> PathBuf {
    PathBuf::from("/var/log/ordin/")
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO Error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Error serializing to TOML {0:?}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("Error deserializing from TOML {0:?}")]
    TomlDe(#[from] toml::de::Error),
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        debug!("Reading configuration from file");
        if !path.exists() {
            trace!(
                "Configuration file does not yet exist, writing default to {:?}",
                path
            );
            return Self::create_default(path);
        }

        trace!("Reading configuration from {:?}", path);
        let mut f = fs::File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        trace!("Deserializing configuration file");
        let this = toml::from_slice(buf.as_slice())?;
        Ok(this)
    }

    fn create_default(path: &Path) -> Result<Self, ConfigError> {
        if let Some(parent) = path.parent() {
            trace!(
                "Configuration path has a parent, creating directories at {:?}",
                parent
            );
            fs::create_dir_all(parent)?;
        }

        trace!("Serializing default configuration");
        let default = Self::default();
        let toml = toml::to_string_pretty(&default)?;

        trace!("Writing default configuration to {:?}", path);
        let mut f = fs::File::create(path)?;
        f.write_all(toml.as_bytes())?;

        warn!(
            "A blank configuration was created. Please configure {} properly.",
            env!("CARGO_PKG_NAME")
        );

        Ok(default)
    }
}

use crate::services::{Service, Target};
use crate::Config;
use log::{debug, trace, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnsibleError {
    #[error("IO error {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Ansible failed")]
    AnsibleFailed,
    #[error("Failed to (de)serialize YAML {0:?}")]
    Yaml(#[from] serde_yaml::Error),
}

#[derive(Debug, Clone)]
pub struct AnsibleService {
    playbooks: Vec<Playbook>,
    inventory: Inventory,
    binary: Option<PathBuf>,
    domain: String,
    play_logdir: PathBuf,
    play_log: bool,
}

#[derive(Debug, Clone)]
pub struct Inventory(PathBuf);

#[derive(Debug, Clone)]
pub struct Playbook(PathBuf);

impl AnsibleService {
    pub fn new(config: &Config) -> Result<Self, AnsibleError> {
        let playbooks = config
            .ansible
            .playbooks
            .iter()
            .map(|x| {
                if !x.exists() {
                    warn!("Ansible playbook {:?} does not exist", x);
                }
                x
            })
            .filter(|x| x.exists())
            .map(|x| Playbook(x.clone()))
            .collect::<Vec<_>>();

        if !config.ansible.inventory.exists() {
            warn!(
                "Inventory {:?} does not exist (It will be created later though)",
                &config.ansible.inventory
            );
        }

        if !config.ansible.play_logdir.exists() {
            trace!(
                "Ansible play logging directory {:?} does not exist, creating",
                config.ansible.play_logdir
            );
            fs::create_dir_all(&config.ansible.play_logdir)?;
        }

        Ok(Self {
            playbooks,
            inventory: Inventory(config.ansible.inventory.clone()),
            binary: config.ansible.ansible_playbook_binary.clone(),
            domain: config.global.domain.clone(),
            play_log: config.ansible.play_logs,
            play_logdir: config.ansible.play_logdir.clone(),
        })
    }
}

mod models {
    use crate::services::ansible::AnsibleError;
    use log::{debug, trace};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct Inventory {
        pub all: All,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct All {
        pub children: HashMap<String, Child>,
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct Child {
        pub hosts: HashMap<String, Option<String>>,
    }

    impl Inventory {
        pub fn create_default(path: &Path) -> Result<Self, AnsibleError> {
            debug!("Creating default inventory at {:?}", path);
            if let Some(parent) = path.parent() {
                debug!("Parent path exists, creating directory tree: {:?}", parent);
                fs::create_dir_all(parent)?;
            }

            trace!("Opening inventory file {:?}", path);
            let mut f = fs::File::create(path)?;
            let this = Self::default();

            trace!("Writing default inventory");
            serde_yaml::to_writer(&mut f, &this)?;
            Ok(this)
        }

        pub fn read(path: &Path) -> Result<Self, AnsibleError> {
            if !path.exists() {
                return Self::create_default(path);
            }

            trace!("Opening inventory file at {:?}", path);
            let f = fs::File::open(path)?;

            trace!("Reading inventory");
            let this: Self = serde_yaml::from_reader(&f)?;
            Ok(this)
        }

        pub fn write(&self, path: &Path) -> Result<(), AnsibleError> {
            trace!("Opening inventory file at {:?}", path);
            let mut f = fs::File::create(path)?;
            trace!("Writing inventory");
            serde_yaml::to_writer(&mut f, self)?;
            Ok(())
        }
    }

    impl Default for All {
        fn default() -> Self {
            let mut children = HashMap::new();
            children.insert("cloud-init".to_string(), Child::default());

            Self { children }
        }
    }
}

impl Service for AnsibleService {
    type Err = AnsibleError;

    fn run(&self, target: &Target) -> Result<(), Self::Err> {
        debug!("Running Ansible service for {:?}", target);

        if !self.is_in_inventory(target)? {
            trace!("Target {:?} is not yet in inventory. Adding", target);
            self.add_to_inventory(target)?;
        }

        let formatted_hostname = self.format_target_name(target);
        for (index, playbook) in self.playbooks.iter().enumerate() {
            trace!(
                "Running Ansible playbook {}/{} for {}",
                index + 1,
                self.playbooks.len(),
                &formatted_hostname
            );
            self.run_playbook(target, playbook)?;
        }

        Ok(())
    }
}

impl AnsibleService {
    fn run_playbook(&self, target: &Target, playbook: &Playbook) -> Result<(), AnsibleError> {
        trace!("Spawning ansible-playbook child process for {:?}", target);

        if !playbook.0.exists() {
            warn!("Ansible playbook {:?} does not exist", &playbook.0);
            return Ok(());
        }

        let child = Command::new(
            self.binary
                .as_deref()
                .unwrap_or(&PathBuf::from("ansible-playbook")),
        )
        .args(&[
            &OsStr::new("-i"),
            &self.inventory.0.as_os_str(),
            &OsStr::new("-l"),
            &OsStr::new(&self.format_target_name(target)),
            &playbook.0.as_os_str(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

        trace!("Waiting for ansible-playbook to complete");
        let output = child.wait_with_output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if self.play_log {
            let path = self.play_logdir.join(format!(
                "{}-ansible_playbook_{}_{}-{}.log",
                time::OffsetDateTime::now_utc().unix_timestamp(),
                playbook.0.file_name().unwrap_or(OsStr::new("")).to_string_lossy(),
                target.ip,
                target.hostname
            ));
            trace!(
                "Ansible playbook logging is enabled. Logging to {:?}",
                &path
            );
            let mut f = fs::File::create(path)?;

            f.write_all(b"STDOUT:\n")?;
            f.write_all(stdout.as_bytes())?;
            f.write_all(b"\nSTDERR:\n")?;
            f.write_all(stderr.as_bytes())?;
        }

        trace!("Ansible stdout: {}", &stdout);
        trace!("Ansible stderr: {}", &stderr);

        if !output.status.success() {
            return Err(AnsibleError::AnsibleFailed);
        }

        trace!("Ansible-playbook completed successfully");

        Ok(())
    }

    fn is_in_inventory(&self, target: &Target) -> Result<bool, AnsibleError> {
        trace!("Checking if target {:?} is in inventory", target);
        let inventory = models::Inventory::read(&self.inventory.0)?;
        let children = inventory
            .all
            .children
            .iter()
            .any(|(_, child)| child.hosts.contains_key(&self.format_target_name(target)));
        Ok(children)
    }

    fn format_target_name(&self, target: &Target) -> String {
        format!("{}.{}", &target.hostname, &self.domain)
    }

    fn add_to_inventory(&self, target: &Target) -> Result<(), AnsibleError> {
        let mut inventory = models::Inventory::read(&self.inventory.0)?;

        inventory
            .all
            .children
            .entry("cloud-init".to_string())
            .and_modify(|child| {
                child.hosts.insert(self.format_target_name(target), None);
            })
            .or_insert_with(|| {
                let mut hosts = HashMap::new();
                hosts.insert(self.format_target_name(target), None);
                models::Child { hosts }
            });

        inventory.write(&self.inventory.0)?;
        Ok(())
    }
}

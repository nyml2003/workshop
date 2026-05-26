use serde::{Deserialize, Serialize};
use std::fs;
use workc_domain::config::{ConfigRepository, WorkcConfig};
use workc_domain::errors::FieldKind;
use workc_domain::errors::DomainError;

use super::paths;

pub struct FsConfigRepository;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigToml {
    #[serde(default)]
    knowledge: KnowledgeConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct KnowledgeConfig {
    remote: Option<String>,
}

impl FsConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsConfigRepository {
    fn default() -> Self {
        Self
    }
}

impl ConfigRepository for FsConfigRepository {
    fn load(&self) -> Result<WorkcConfig, DomainError> {
        let path = paths::workc_config_path();
        if !path.exists() {
            return Ok(WorkcConfig::default());
        }
        let raw = fs::read_to_string(&path).map_err(io_error("read config"))?;
        let config: ConfigToml = toml::from_str(&raw).map_err(invalid_toml("config.toml"))?;
        Ok(WorkcConfig {
            knowledge_remote: config.knowledge.remote,
        })
    }

    fn save(&self, config: &WorkcConfig) -> Result<(), DomainError> {
        let path = paths::workc_config_path();
        let parent = path.parent().ok_or(DomainError::InvalidInput {
            field: FieldKind::Other("config path"),
            reason: "no parent directory".to_owned(),
        })?;
        fs::create_dir_all(parent).map_err(io_error("create workc home"))?;
        let toml = ConfigToml {
            knowledge: KnowledgeConfig {
                remote: config.knowledge_remote.clone(),
            },
        };
        fs::write(
            &path,
            toml::to_string_pretty(&toml).map_err(invalid_serialize("config.toml"))?,
        )
        .map_err(io_error("write config"))?;
        Ok(())
    }
}

fn io_error(operation: &'static str) -> impl Fn(std::io::Error) -> DomainError { move |error| DomainError::PersistenceFailed { operation: operation,
        detail: error.to_string(),
    }
}

fn invalid_toml(field: &'static str) -> impl Fn(toml::de::Error) -> DomainError { move |error| DomainError::InvalidInput { field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}

fn invalid_serialize(field: &'static str) -> impl Fn(toml::ser::Error) -> DomainError { move |error| DomainError::InvalidInput { field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}


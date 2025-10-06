use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::error::{CocoatlyError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub registry: RegistryConfig,
    pub storage: StorageConfig,
    pub cache: CacheConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub default_registry: String,
    pub registries: HashMap<String, RegistryEndpoint>,
    pub auth_tokens: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEndpoint {
    pub url: String,
    pub api_version: String,
    pub requires_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub install_root: PathBuf,
    pub cache_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub state_file: PathBuf,
    pub lock_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub ttl_hours: u64,
    pub cleanup_on_startup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_concurrent_downloads: usize,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub use_proxy: bool,
    pub proxy_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub verify_signatures: bool,
    pub verify_checksums: bool,
    pub allowed_hash_algorithms: Vec<String>,
    pub trusted_keys: Vec<String>,
    pub reject_insecure_registries: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HooksConfig {
    pub pre_install: Vec<String>,
    pub post_install: Vec<String>,
    pub pre_uninstall: Vec<String>,
    pub post_uninstall: Vec<String>,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn default() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/tmp".to_string());

        let cocoatly_home = PathBuf::from(home_dir).join(".cocoatly");

        Self {
            registry: RegistryConfig {
                default_registry: "cocoatly-registry".to_string(),
                registries: HashMap::from([
                    ("cocoatly-registry".to_string(), RegistryEndpoint {
                        url: "https://registry.cocoatly.io".to_string(),
                        api_version: "v1".to_string(),
                        requires_auth: false,
                    })
                ]),
                auth_tokens: HashMap::new(),
            },
            storage: StorageConfig {
                install_root: cocoatly_home.join("packages"),
                cache_dir: cocoatly_home.join("cache"),
                temp_dir: cocoatly_home.join("tmp"),
                state_file: cocoatly_home.join("state.json"),
                lock_file: cocoatly_home.join("cocoatly.lock"),
            },
            cache: CacheConfig {
                enabled: true,
                max_size_mb: 5120,
                ttl_hours: 168,
                cleanup_on_startup: false,
            },
            network: NetworkConfig {
                max_concurrent_downloads: 8,
                timeout_seconds: 300,
                retry_attempts: 3,
                retry_delay_ms: 1000,
                use_proxy: false,
                proxy_url: None,
            },
            security: SecurityConfig {
                verify_signatures: true,
                verify_checksums: true,
                allowed_hash_algorithms: vec![
                    "blake3".to_string(),
                    "sha256".to_string(),
                    "sha512".to_string(),
                ],
                trusted_keys: vec![],
                reject_insecure_registries: true,
            },
            hooks: HooksConfig {
                pre_install: vec![],
                post_install: vec![],
                pre_uninstall: vec![],
                post_uninstall: vec![],
            },
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.registry.default_registry.is_empty() {
            return Err(CocoatlyError::ConfigError(
                "Default registry not set".to_string()
            ));
        }

        if !self.registry.registries.contains_key(&self.registry.default_registry) {
            return Err(CocoatlyError::ConfigError(
                "Default registry not found in registry list".to_string()
            ));
        }

        if self.network.max_concurrent_downloads == 0 {
            return Err(CocoatlyError::ConfigError(
                "max_concurrent_downloads must be greater than 0".to_string()
            ));
        }

        Ok(())
    }
}

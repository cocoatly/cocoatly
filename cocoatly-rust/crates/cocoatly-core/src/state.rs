use crate::types::{InstalledPackage, PackageName, Version};
use crate::error::{CocoatlyError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalState {
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub installed_packages: HashMap<PackageName, InstalledPackage>,
    pub pending_operations: Vec<String>,
    pub metadata: StateMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    pub total_packages: usize,
    pub total_size_bytes: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub corrupted_packages: Vec<String>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: Utc::now(),
            installed_packages: HashMap::new(),
            pending_operations: vec![],
            metadata: StateMetadata {
                total_packages: 0,
                total_size_bytes: 0,
                last_cleanup: None,
                corrupted_packages: vec![],
            },
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)?;
        let state: GlobalState = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_package(&mut self, package: InstalledPackage) {
        self.installed_packages.insert(package.name.clone(), package);
        self.metadata.total_packages = self.installed_packages.len();
        self.last_updated = Utc::now();
    }

    pub fn remove_package(&mut self, name: &PackageName) -> Option<InstalledPackage> {
        let removed = self.installed_packages.remove(name);
        if removed.is_some() {
            self.metadata.total_packages = self.installed_packages.len();
            self.last_updated = Utc::now();
        }
        removed
    }

    pub fn get_package(&self, name: &PackageName) -> Option<&InstalledPackage> {
        self.installed_packages.get(name)
    }

    pub fn has_package(&self, name: &PackageName, version: &Version) -> bool {
        self.installed_packages
            .get(name)
            .map(|pkg| &pkg.version == version)
            .unwrap_or(false)
    }

    pub fn list_packages(&self) -> Vec<&InstalledPackage> {
        self.installed_packages.values().collect()
    }

    pub fn update_metadata(&mut self) {
        self.metadata.total_packages = self.installed_packages.len();
        self.metadata.total_size_bytes = self.installed_packages
            .values()
            .map(|pkg| {
                pkg.files.iter()
                    .filter_map(|path| std::fs::metadata(path).ok())
                    .map(|m| m.len())
                    .sum::<u64>()
            })
            .sum();
        self.last_updated = Utc::now();
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

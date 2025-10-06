use cocoatly_core::{
    types::{PackageName, Version, InstalledPackage},
    error::{CocoatlyError, Result},
    config::Config,
    state::GlobalState,
};
use cocoatly_fs::FileSystemOps;
use std::path::PathBuf;

pub struct PackageUninstaller {
    config: Config,
    state: GlobalState,
}

impl PackageUninstaller {
    pub fn new(config: Config, state: GlobalState) -> Self {
        Self { config, state }
    }

    pub fn uninstall(&mut self, name: &PackageName) -> Result<()> {
        tracing::info!("Uninstalling package {}", name.as_str());

        let package = self.state
            .get_package(name)
            .ok_or_else(|| CocoatlyError::PackageNotFound(name.as_str().to_string()))?
            .clone();

        self.run_pre_uninstall_hooks(&package)?;

        self.remove_package_files(&package)?;

        self.state.remove_package(name);
        self.state.save_to_file(&self.config.storage.state_file)?;

        self.run_post_uninstall_hooks(&package)?;

        tracing::info!("Successfully uninstalled package {}", name.as_str());

        Ok(())
    }

    fn remove_package_files(&self, package: &InstalledPackage) -> Result<()> {
        let install_path = PathBuf::from(&package.install_path);

        if install_path.exists() {
            tracing::info!("Removing package files from {}", install_path.display());
            FileSystemOps::remove_directory(&install_path)?;
        }

        let package_dir = self.config.storage.install_root.join(package.name.as_str());
        if package_dir.exists() && FileSystemOps::list_files(&package_dir)?.is_empty() {
            FileSystemOps::remove_directory(&package_dir)?;
        }

        Ok(())
    }

    fn run_pre_uninstall_hooks(&self, package: &InstalledPackage) -> Result<()> {
        for hook in &self.config.hooks.pre_uninstall {
            tracing::info!("Running pre-uninstall hook: {}", hook);
        }
        Ok(())
    }

    fn run_post_uninstall_hooks(&self, package: &InstalledPackage) -> Result<()> {
        for hook in &self.config.hooks.post_uninstall {
            tracing::info!("Running post-uninstall hook: {}", hook);
        }
        Ok(())
    }

    pub fn force_uninstall(&mut self, name: &PackageName) -> Result<()> {
        tracing::warn!("Force uninstalling package {}", name.as_str());

        if let Some(package) = self.state.get_package(name).cloned() {
            let install_path = PathBuf::from(&package.install_path);
            if install_path.exists() {
                FileSystemOps::remove_directory(&install_path)?;
            }
        }

        self.state.remove_package(name);
        self.state.save_to_file(&self.config.storage.state_file)?;

        Ok(())
    }
}

pub fn uninstall_package(
    config: Config,
    state: GlobalState,
    name: &PackageName,
) -> Result<()> {
    let mut uninstaller = PackageUninstaller::new(config, state);
    uninstaller.uninstall(name)
}

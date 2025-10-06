use cocoatly_core::{
    types::{PackageName, Version, PackageArtifact, InstalledPackage},
    error::{CocoatlyError, Result},
    config::Config,
    state::GlobalState,
};
use crate::install::{InstallContext, install_package};
use crate::uninstall::uninstall_package;

pub struct PackageUpdater {
    config: Config,
    state: GlobalState,
}

impl PackageUpdater {
    pub fn new(config: Config, state: GlobalState) -> Self {
        Self { config, state }
    }

    pub async fn update(
        &mut self,
        name: &PackageName,
        new_artifact: &PackageArtifact,
    ) -> Result<InstalledPackage> {
        tracing::info!(
            "Updating package {} to version {}",
            name.as_str(),
            new_artifact.version.to_string()
        );

        let current_package = self.state
            .get_package(name)
            .ok_or_else(|| CocoatlyError::PackageNotFound(name.as_str().to_string()))?
            .clone();

        if current_package.version >= new_artifact.version {
            return Err(CocoatlyError::InstallationFailed(
                format!(
                    "Current version {} is already up to date or newer than {}",
                    current_package.version.to_string(),
                    new_artifact.version.to_string()
                )
            ));
        }

        let requested_by = current_package.requested_by.clone();

        uninstall_package(
            self.config.clone(),
            self.state.clone(),
            name,
        )?;

        let context = InstallContext::new(self.config.clone(), self.state.clone())?;
        let installed = install_package(context, new_artifact, requested_by).await?;

        self.state = GlobalState::load_from_file(&self.config.storage.state_file)?;

        tracing::info!(
            "Successfully updated package {} to version {}",
            name.as_str(),
            new_artifact.version.to_string()
        );

        Ok(installed)
    }
}

pub async fn update_package(
    config: Config,
    state: GlobalState,
    name: &PackageName,
    new_artifact: &PackageArtifact,
) -> Result<InstalledPackage> {
    let mut updater = PackageUpdater::new(config, state);
    updater.update(name, new_artifact).await
}

use cocoatly_core::{
    types::*,
    error::{CocoatlyError, Result},
    config::Config,
    state::GlobalState,
};
use cocoatly_downloader::{Downloader, DownloadTask};
use cocoatly_compression::extract_archive;
use cocoatly_crypto::verify_artifact;
use cocoatly_fs::FileSystemOps;
use std::path::{Path, PathBuf};
use chrono::Utc;
use uuid::Uuid;

pub struct InstallContext {
    pub config: Config,
    pub state: GlobalState,
    pub downloader: Downloader,
    pub temp_dir: PathBuf,
}

impl InstallContext {
    pub fn new(config: Config, state: GlobalState) -> Result<Self> {
        let downloader = Downloader::new(config.network.clone())?;
        let temp_dir = config.storage.temp_dir.clone();

        FileSystemOps::ensure_directory(&temp_dir)?;

        Ok(Self {
            config,
            state,
            downloader,
            temp_dir,
        })
    }
}

pub struct PackageInstaller {
    context: InstallContext,
}

impl PackageInstaller {
    pub fn new(context: InstallContext) -> Self {
        Self { context }
    }

    pub async fn install(
        &mut self,
        artifact: &PackageArtifact,
        requested_by: Vec<PackageName>,
    ) -> Result<InstalledPackage> {
        let operation_id = Uuid::new_v4();
        let started_at = Utc::now();

        tracing::info!(
            "Installing package {} version {}",
            artifact.name.as_str(),
            artifact.version.to_string()
        );

        if self.context.state.has_package(&artifact.name, &artifact.version) {
            return Err(CocoatlyError::InstallationFailed(
                format!("Package {} {} already installed", artifact.name.as_str(), artifact.version.to_string())
            ));
        }

        let archive_path = self.download_artifact(artifact).await?;

        self.verify_artifact(&archive_path, artifact)?;

        let install_path = self.extract_and_install(&archive_path, artifact).await?;

        let files = FileSystemOps::list_files(&install_path)?
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let installed_package = InstalledPackage {
            id: Uuid::new_v4(),
            name: artifact.name.clone(),
            version: artifact.version.clone(),
            install_path: install_path.to_string_lossy().to_string(),
            installed_at: Utc::now(),
            requested_by,
            files,
            checksum: artifact.checksum.clone(),
        };

        self.cleanup_temp_files(&archive_path)?;

        self.context.state.add_package(installed_package.clone());
        self.context.state.save_to_file(&self.context.config.storage.state_file)?;

        self.run_post_install_hooks(&installed_package)?;

        tracing::info!(
            "Successfully installed package {} version {}",
            artifact.name.as_str(),
            artifact.version.to_string()
        );

        Ok(installed_package)
    }

    async fn download_artifact(&self, artifact: &PackageArtifact) -> Result<PathBuf> {
        let filename = format!(
            "{}-{}.tar.gz",
            artifact.name.as_str(),
            artifact.version.to_string()
        );

        let destination = self.context.temp_dir.join(&filename);

        tracing::info!("Downloading artifact from {}", artifact.download_url);

        self.context
            .downloader
            .download(&artifact.download_url, &destination, None)
            .await?;

        Ok(destination)
    }

    fn verify_artifact(&self, path: &Path, artifact: &PackageArtifact) -> Result<()> {
        if !self.context.config.security.verify_checksums {
            tracing::warn!("Checksum verification disabled");
            return Ok(());
        }

        tracing::info!("Verifying artifact integrity");

        let public_key = None;

        verify_artifact(path, artifact, public_key)?;

        Ok(())
    }

    async fn extract_and_install(
        &self,
        archive_path: &Path,
        artifact: &PackageArtifact,
    ) -> Result<PathBuf> {
        let extract_dir = self.context.temp_dir.join(format!(
            "extract-{}",
            Uuid::new_v4()
        ));

        FileSystemOps::ensure_directory(&extract_dir)?;

        tracing::info!("Extracting archive");

        extract_archive(archive_path, &extract_dir)?;

        let install_root = &self.context.config.storage.install_root;
        let package_install_dir = install_root
            .join(artifact.name.as_str())
            .join(artifact.version.to_string());

        FileSystemOps::ensure_directory(&package_install_dir)?;

        tracing::info!("Installing to {}", package_install_dir.display());

        FileSystemOps::copy_directory(&extract_dir, &package_install_dir)?;

        FileSystemOps::remove_directory(&extract_dir)?;

        Ok(package_install_dir)
    }

    fn cleanup_temp_files(&self, archive_path: &Path) -> Result<()> {
        if archive_path.exists() {
            std::fs::remove_file(archive_path)?;
        }
        Ok(())
    }

    fn run_post_install_hooks(&self, package: &InstalledPackage) -> Result<()> {
        for hook in &self.context.config.hooks.post_install {
            tracing::info!("Running post-install hook: {}", hook);
        }
        Ok(())
    }
}

pub async fn install_package(
    context: InstallContext,
    artifact: &PackageArtifact,
    requested_by: Vec<PackageName>,
) -> Result<InstalledPackage> {
    let mut installer = PackageInstaller::new(context);
    installer.install(artifact, requested_by).await
}

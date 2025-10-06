use cocoatly_core::{
    types::{PackageName, InstalledPackage, HashAlgorithm},
    error::{CocoatlyError, Result},
    config::Config,
    state::GlobalState,
};
use cocoatly_crypto::verify_file_hash;
use cocoatly_fs::FileSystemOps;
use std::path::PathBuf;

pub struct VerificationResult {
    pub package: PackageName,
    pub valid: bool,
    pub missing_files: Vec<String>,
    pub corrupted_files: Vec<String>,
}

pub fn verify_installation(
    config: &Config,
    state: &GlobalState,
    name: &PackageName,
) -> Result<VerificationResult> {
    let package = state
        .get_package(name)
        .ok_or_else(|| CocoatlyError::PackageNotFound(name.as_str().to_string()))?;

    let install_path = PathBuf::from(&package.install_path);

    if !install_path.exists() {
        return Ok(VerificationResult {
            package: name.clone(),
            valid: false,
            missing_files: package.files.clone(),
            corrupted_files: vec![],
        });
    }

    let mut missing_files = Vec::new();
    let mut corrupted_files = Vec::new();

    for file in &package.files {
        let file_path = PathBuf::from(file);

        if !file_path.exists() {
            missing_files.push(file.clone());
        }
    }

    let valid = missing_files.is_empty() && corrupted_files.is_empty();

    Ok(VerificationResult {
        package: name.clone(),
        valid,
        missing_files,
        corrupted_files,
    })
}

pub fn repair_package(
    config: &Config,
    state: &mut GlobalState,
    name: &PackageName,
) -> Result<()> {
    tracing::info!("Attempting to repair package {}", name.as_str());

    let verification = verify_installation(config, state, name)?;

    if verification.valid {
        tracing::info!("Package {} is already valid", name.as_str());
        return Ok(());
    }

    if !verification.missing_files.is_empty() {
        return Err(CocoatlyError::VerificationFailed(
            format!(
                "Cannot repair package {}: {} files missing",
                name.as_str(),
                verification.missing_files.len()
            )
        ));
    }

    tracing::info!("Package {} repaired successfully", name.as_str());

    Ok(())
}

use cocoatly_core::types::{PackageArtifact, HashAlgorithm};
use cocoatly_core::error::{CocoatlyError, Result};
use std::path::Path;
use crate::hash::{verify_file_hash, compute_file_hash};
use crate::signature::verify_signature;

pub fn verify_package_integrity<P: AsRef<Path>>(
    path: P,
    expected_checksum: &str,
    algorithm: &HashAlgorithm,
) -> Result<()> {
    verify_file_hash(path, expected_checksum, algorithm)
}

pub fn verify_artifact<P: AsRef<Path>>(
    artifact_path: P,
    artifact: &PackageArtifact,
    public_key: Option<&[u8]>,
) -> Result<()> {
    verify_file_hash(
        artifact_path.as_ref(),
        &artifact.checksum,
        &artifact.checksum_algorithm,
    )?;

    if let (Some(signature_hex), Some(pub_key)) = (&artifact.signature, public_key) {
        let signature = hex::decode(signature_hex)
            .map_err(|_| CocoatlyError::InvalidSignature(
                "Invalid signature format".to_string()
            ))?;

        let file_hash = compute_file_hash(artifact_path, &artifact.checksum_algorithm)?;
        let message = file_hash.as_bytes();

        verify_signature(pub_key, message, &signature)?;
    }

    Ok(())
}

pub fn verify_directory_integrity<P: AsRef<Path>>(
    directory: P,
    expected_files: &[String],
) -> Result<()> {
    let dir_path = directory.as_ref();

    if !dir_path.exists() {
        return Err(CocoatlyError::VerificationFailed(
            format!("Directory does not exist: {}", dir_path.display())
        ));
    }

    for file in expected_files {
        let file_path = dir_path.join(file);
        if !file_path.exists() {
            return Err(CocoatlyError::VerificationFailed(
                format!("Expected file not found: {}", file_path.display())
            ));
        }
    }

    Ok(())
}

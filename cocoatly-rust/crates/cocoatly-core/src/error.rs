use std::fmt;

#[derive(Debug)]
pub enum CocoatlyError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    PackageNotFound(String),
    VersionConflict(String),
    InvalidManifest(String),
    DownloadFailed(String),
    VerificationFailed(String),
    InstallationFailed(String),
    DependencyResolutionFailed(String),
    InvalidSignature(String),
    HashMismatch { expected: String, actual: String },
    PermissionDenied(String),
    ConfigError(String),
    StateError(String),
    RegistryError(String),
}

impl fmt::Display for CocoatlyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CocoatlyError::IoError(e) => write!(f, "IO error: {}", e),
            CocoatlyError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            CocoatlyError::PackageNotFound(pkg) => write!(f, "Package not found: {}", pkg),
            CocoatlyError::VersionConflict(msg) => write!(f, "Version conflict: {}", msg),
            CocoatlyError::InvalidManifest(msg) => write!(f, "Invalid manifest: {}", msg),
            CocoatlyError::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            CocoatlyError::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
            CocoatlyError::InstallationFailed(msg) => write!(f, "Installation failed: {}", msg),
            CocoatlyError::DependencyResolutionFailed(msg) => write!(f, "Dependency resolution failed: {}", msg),
            CocoatlyError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            CocoatlyError::HashMismatch { expected, actual } => {
                write!(f, "Hash mismatch: expected {}, got {}", expected, actual)
            }
            CocoatlyError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            CocoatlyError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CocoatlyError::StateError(msg) => write!(f, "State error: {}", msg),
            CocoatlyError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for CocoatlyError {}

impl From<std::io::Error> for CocoatlyError {
    fn from(err: std::io::Error) -> Self {
        CocoatlyError::IoError(err)
    }
}

impl From<serde_json::Error> for CocoatlyError {
    fn from(err: serde_json::Error) -> Self {
        CocoatlyError::SerializationError(err)
    }
}

pub type Result<T> = std::result::Result<T, CocoatlyError>;

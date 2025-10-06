use crate::types::{PackageManifest, PackageMetadata, Dependency, PackageName, Version};
use crate::error::{CocoatlyError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct ManifestFile {
    package: PackageSection,
    dependencies: Option<HashMap<String, String>>,
    dev_dependencies: Option<HashMap<String, String>>,
    build_dependencies: Option<HashMap<String, String>>,
    scripts: Option<HashMap<String, String>>,
    features: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageSection {
    name: String,
    version: String,
    description: Option<String>,
    authors: Option<Vec<String>>,
    license: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    keywords: Option<Vec<String>>,
    categories: Option<Vec<String>>,
}

pub fn load_manifest<P: AsRef<Path>>(path: P) -> Result<PackageManifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest_file: ManifestFile = serde_json::from_str(&content)?;

    let version = Version::parse(&manifest_file.package.version)
        .ok_or_else(|| CocoatlyError::InvalidManifest(
            format!("Invalid version: {}", manifest_file.package.version)
        ))?;

    let metadata = PackageMetadata {
        id: uuid::Uuid::new_v4(),
        name: PackageName::new(manifest_file.package.name),
        version,
        description: manifest_file.package.description,
        authors: manifest_file.package.authors.unwrap_or_default(),
        license: manifest_file.package.license,
        homepage: manifest_file.package.homepage,
        repository: manifest_file.package.repository,
        keywords: manifest_file.package.keywords.unwrap_or_default(),
        categories: manifest_file.package.categories.unwrap_or_default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let dependencies = parse_dependencies(manifest_file.dependencies)?;
    let dev_dependencies = parse_dependencies(manifest_file.dev_dependencies)?;
    let build_dependencies = parse_dependencies(manifest_file.build_dependencies)?;

    Ok(PackageManifest {
        package: metadata,
        dependencies,
        dev_dependencies,
        build_dependencies,
        scripts: manifest_file.scripts.unwrap_or_default(),
        features: manifest_file.features.unwrap_or_default(),
    })
}

fn parse_dependencies(deps: Option<HashMap<String, String>>) -> Result<Vec<Dependency>> {
    let Some(deps) = deps else {
        return Ok(vec![]);
    };

    let mut result = Vec::new();

    for (name, version_req) in deps {
        let version_requirement = parse_version_requirement(&version_req)?;
        result.push(Dependency {
            name: PackageName::new(name),
            version_requirement,
            optional: false,
            features: vec![],
        });
    }

    Ok(result)
}

fn parse_version_requirement(req: &str) -> Result<crate::types::VersionRequirement> {
    use crate::types::VersionRequirement;

    let req = req.trim();

    if req == "*" || req.is_empty() {
        return Ok(VersionRequirement::Any);
    }

    if let Some(version_str) = req.strip_prefix("^") {
        let version = Version::parse(version_str)
            .ok_or_else(|| CocoatlyError::InvalidManifest(
                format!("Invalid version requirement: {}", req)
            ))?;
        return Ok(VersionRequirement::Compatible(version));
    }

    if let Some(version_str) = req.strip_prefix(">=") {
        let version = Version::parse(version_str.trim())
            .ok_or_else(|| CocoatlyError::InvalidManifest(
                format!("Invalid version requirement: {}", req)
            ))?;
        return Ok(VersionRequirement::GreaterThanOrEqual(version));
    }

    if let Some(version_str) = req.strip_prefix(">") {
        let version = Version::parse(version_str.trim())
            .ok_or_else(|| CocoatlyError::InvalidManifest(
                format!("Invalid version requirement: {}", req)
            ))?;
        return Ok(VersionRequirement::GreaterThan(version));
    }

    if let Some(version_str) = req.strip_prefix("<=") {
        let version = Version::parse(version_str.trim())
            .ok_or_else(|| CocoatlyError::InvalidManifest(
                format!("Invalid version requirement: {}", req)
            ))?;
        return Ok(VersionRequirement::LessThanOrEqual(version));
    }

    if let Some(version_str) = req.strip_prefix("<") {
        let version = Version::parse(version_str.trim())
            .ok_or_else(|| CocoatlyError::InvalidManifest(
                format!("Invalid version requirement: {}", req)
            ))?;
        return Ok(VersionRequirement::LessThan(version));
    }

    let version = Version::parse(req)
        .ok_or_else(|| CocoatlyError::InvalidManifest(
            format!("Invalid version requirement: {}", req)
        ))?;
    Ok(VersionRequirement::Exact(version))
}

pub fn save_manifest<P: AsRef<Path>>(manifest: &PackageManifest, path: P) -> Result<()> {
    let package_section = PackageSection {
        name: manifest.package.name.0.clone(),
        version: manifest.package.version.to_string(),
        description: manifest.package.description.clone(),
        authors: Some(manifest.package.authors.clone()),
        license: manifest.package.license.clone(),
        homepage: manifest.package.homepage.clone(),
        repository: manifest.package.repository.clone(),
        keywords: Some(manifest.package.keywords.clone()),
        categories: Some(manifest.package.categories.clone()),
    };

    let manifest_file = ManifestFile {
        package: package_section,
        dependencies: Some(deps_to_map(&manifest.dependencies)),
        dev_dependencies: Some(deps_to_map(&manifest.dev_dependencies)),
        build_dependencies: Some(deps_to_map(&manifest.build_dependencies)),
        scripts: Some(manifest.scripts.clone()),
        features: Some(manifest.features.clone()),
    };

    let content = serde_json::to_string_pretty(&manifest_file)?;
    std::fs::write(path, content)?;
    Ok(())
}

fn deps_to_map(deps: &[Dependency]) -> HashMap<String, String> {
    deps.iter()
        .map(|d| (d.name.0.clone(), version_req_to_string(&d.version_requirement)))
        .collect()
}

fn version_req_to_string(req: &crate::types::VersionRequirement) -> String {
    use crate::types::VersionRequirement;

    match req {
        VersionRequirement::Exact(v) => v.to_string(),
        VersionRequirement::Compatible(v) => format!("^{}", v.to_string()),
        VersionRequirement::GreaterThan(v) => format!(">{}", v.to_string()),
        VersionRequirement::GreaterThanOrEqual(v) => format!(">={}", v.to_string()),
        VersionRequirement::LessThan(v) => format!("<{}", v.to_string()),
        VersionRequirement::LessThanOrEqual(v) => format!("<={}", v.to_string()),
        VersionRequirement::Any => "*".to_string(),
        VersionRequirement::Range { min, max } => {
            format!(">={},<{}", min.to_string(), max.to_string())
        }
    }
}

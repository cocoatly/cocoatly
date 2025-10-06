import requests
from typing import List, Dict, Any, Optional
from ..core.models import Package, PackageVersion, PackageMetadata, Version, Dependency, VersionRequirement
from ..core.exceptions import RegistryException, PackageNotFoundException
from ..core.config import CocoatlyConfig

class RegistryClient:
    def __init__(self, config: CocoatlyConfig):
        self.config = config
        self.registry_config = config.registry
        self.base_url = self._get_registry_url()
        self.session = requests.Session()
        self._setup_auth()

    def _get_registry_url(self) -> str:
        default_registry = self.registry_config.default_registry
        registry = self.registry_config.registries.get(default_registry)

        if not registry:
            raise RegistryException(f"Registry '{default_registry}' not found in configuration")

        return registry.url

    def _setup_auth(self) -> None:
        default_registry = self.registry_config.default_registry
        auth_token = self.registry_config.auth_tokens.get(default_registry)

        if auth_token:
            self.session.headers.update({
                "Authorization": f"Bearer {auth_token}"
            })

    def search_packages(
        self,
        query: str,
        limit: int = 50,
        offset: int = 0,
        categories: Optional[List[str]] = None,
        keywords: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        endpoint = f"{self.base_url}/api/v1/packages/search"

        params = {
            "q": query,
            "limit": limit,
            "offset": offset,
        }

        if categories:
            params["categories"] = ",".join(categories)
        if keywords:
            params["keywords"] = ",".join(keywords)

        try:
            response = self.session.get(endpoint, params=params)
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            raise RegistryException(f"Failed to search packages: {str(e)}")

    def get_package(self, package_name: str) -> Package:
        endpoint = f"{self.base_url}/api/v1/packages/{package_name}"

        try:
            response = self.session.get(endpoint)
            response.raise_for_status()
            data = response.json()
            return self._parse_package(data)
        except requests.HTTPError as e:
            if e.response.status_code == 404:
                raise PackageNotFoundException(package_name)
            raise RegistryException(f"Failed to get package '{package_name}': {str(e)}")
        except requests.RequestException as e:
            raise RegistryException(f"Failed to get package '{package_name}': {str(e)}")

    def get_package_versions(self, package_name: str) -> List[PackageVersion]:
        endpoint = f"{self.base_url}/api/v1/packages/{package_name}/versions"

        try:
            response = self.session.get(endpoint)
            response.raise_for_status()
            data = response.json()
            return [self._parse_package_version(v) for v in data.get("versions", [])]
        except requests.RequestException as e:
            raise RegistryException(f"Failed to get versions for '{package_name}': {str(e)}")

    def get_package_version(self, package_name: str, version: str) -> PackageVersion:
        endpoint = f"{self.base_url}/api/v1/packages/{package_name}/versions/{version}"

        try:
            response = self.session.get(endpoint)
            response.raise_for_status()
            data = response.json()
            return self._parse_package_version(data)
        except requests.HTTPError as e:
            if e.response.status_code == 404:
                raise PackageNotFoundException(f"{package_name}@{version}")
            raise RegistryException(f"Failed to get version '{version}' of '{package_name}': {str(e)}")
        except requests.RequestException as e:
            raise RegistryException(f"Failed to get version '{version}' of '{package_name}': {str(e)}")

    def publish_package(
        self,
        package: Package,
        artifact_path: str,
        checksum: str,
        checksum_algorithm: str,
    ) -> Dict[str, Any]:
        endpoint = f"{self.base_url}/api/v1/packages/publish"

        files = {
            "artifact": open(artifact_path, "rb"),
        }

        data = {
            "package_json": self._serialize_package(package),
            "checksum": checksum,
            "checksum_algorithm": checksum_algorithm,
        }

        try:
            response = self.session.post(endpoint, data=data, files=files)
            response.raise_for_status()
            return response.json()
        except requests.RequestException as e:
            raise RegistryException(f"Failed to publish package: {str(e)}")
        finally:
            files["artifact"].close()

    def record_download(self, package_name: str, version: str) -> None:
        endpoint = f"{self.base_url}/api/v1/stats/download"

        data = {
            "package": package_name,
            "version": version,
        }

        try:
            self.session.post(endpoint, json=data)
        except requests.RequestException:
            pass

    def _parse_package(self, data: Dict[str, Any]) -> Package:
        metadata = PackageMetadata(
            id=data["id"],
            name=data["name"],
            version=Version.parse(data["version"]),
            description=data.get("description"),
            authors=data.get("authors", []),
            license=data.get("license"),
            homepage=data.get("homepage"),
            repository=data.get("repository"),
            keywords=data.get("keywords", []),
            categories=data.get("categories", []),
        )

        dependencies = [
            Dependency(
                name=dep["name"],
                version_requirement=VersionRequirement(dep["version_requirement"]),
                optional=dep.get("optional", False),
                features=dep.get("features", []),
            )
            for dep in data.get("dependencies", [])
        ]

        return Package(
            metadata=metadata,
            dependencies=dependencies,
            scripts=data.get("scripts", {}),
            features=data.get("features", {}),
        )

    def _parse_package_version(self, data: Dict[str, Any]) -> PackageVersion:
        return PackageVersion(
            package_name=data["package_name"],
            version=Version.parse(data["version"]),
            download_url=data["download_url"],
            checksum=data["checksum"],
            checksum_algorithm=data["checksum_algorithm"],
            signature=data.get("signature"),
            size=data.get("size", 0),
        )

    def _serialize_package(self, package: Package) -> str:
        import json

        data = {
            "name": package.metadata.name,
            "version": str(package.metadata.version),
            "description": package.metadata.description,
            "authors": package.metadata.authors,
            "license": package.metadata.license,
            "homepage": package.metadata.homepage,
            "repository": package.metadata.repository,
            "keywords": package.metadata.keywords,
            "categories": package.metadata.categories,
            "dependencies": [
                {
                    "name": dep.name,
                    "version_requirement": dep.version_requirement.requirement_str,
                    "optional": dep.optional,
                    "features": dep.features,
                }
                for dep in package.dependencies
            ],
            "scripts": package.scripts,
            "features": package.features,
        }

        return json.dumps(data)

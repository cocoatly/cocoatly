import json
import os
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass, field, asdict

@dataclass
class RegistryEndpoint:
    url: str
    api_version: str
    requires_auth: bool

@dataclass
class RegistryConfig:
    default_registry: str
    registries: Dict[str, RegistryEndpoint]
    auth_tokens: Dict[str, str] = field(default_factory=dict)

@dataclass
class StorageConfig:
    install_root: Path
    cache_dir: Path
    temp_dir: Path
    state_file: Path
    lock_file: Path

@dataclass
class CacheConfig:
    enabled: bool = True
    max_size_mb: int = 5120
    ttl_hours: int = 168
    cleanup_on_startup: bool = False

@dataclass
class NetworkConfig:
    max_concurrent_downloads: int = 8
    timeout_seconds: int = 300
    retry_attempts: int = 3
    retry_delay_ms: int = 1000
    use_proxy: bool = False
    proxy_url: Optional[str] = None

@dataclass
class SecurityConfig:
    verify_signatures: bool = True
    verify_checksums: bool = True
    allowed_hash_algorithms: List[str] = field(default_factory=lambda: ["blake3", "sha256", "sha512"])
    trusted_keys: List[str] = field(default_factory=list)
    reject_insecure_registries: bool = True

@dataclass
class HooksConfig:
    pre_install: List[str] = field(default_factory=list)
    post_install: List[str] = field(default_factory=list)
    pre_uninstall: List[str] = field(default_factory=list)
    post_uninstall: List[str] = field(default_factory=list)

@dataclass
class CocoatlyConfig:
    registry: RegistryConfig
    storage: StorageConfig
    cache: CacheConfig
    network: NetworkConfig
    security: SecurityConfig
    hooks: HooksConfig

    @classmethod
    def load(cls, config_path: Optional[Path] = None) -> "CocoatlyConfig":
        if config_path is None:
            config_path = cls.default_config_path()

        if not config_path.exists():
            config = cls.default()
            config.save(config_path)
            return config

        with open(config_path, "r") as f:
            data = json.load(f)

        return cls.from_dict(data)

    @classmethod
    def from_dict(cls, data: dict) -> "CocoatlyConfig":
        registry_data = data.get("registry", {})
        registries = {
            name: RegistryEndpoint(**endpoint_data)
            for name, endpoint_data in registry_data.get("registries", {}).items()
        }

        registry = RegistryConfig(
            default_registry=registry_data.get("default_registry", "cocoatly-registry"),
            registries=registries,
            auth_tokens=registry_data.get("auth_tokens", {}),
        )

        storage_data = data.get("storage", {})
        storage = StorageConfig(
            install_root=Path(storage_data.get("install_root")),
            cache_dir=Path(storage_data.get("cache_dir")),
            temp_dir=Path(storage_data.get("temp_dir")),
            state_file=Path(storage_data.get("state_file")),
            lock_file=Path(storage_data.get("lock_file")),
        )

        cache = CacheConfig(**data.get("cache", {}))
        network = NetworkConfig(**data.get("network", {}))
        security = SecurityConfig(**data.get("security", {}))
        hooks = HooksConfig(**data.get("hooks", {}))

        return cls(
            registry=registry,
            storage=storage,
            cache=cache,
            network=network,
            security=security,
            hooks=hooks,
        )

    def to_dict(self) -> dict:
        config_dict = {
            "registry": {
                "default_registry": self.registry.default_registry,
                "registries": {
                    name: asdict(endpoint)
                    for name, endpoint in self.registry.registries.items()
                },
                "auth_tokens": self.registry.auth_tokens,
            },
            "storage": {
                "install_root": str(self.storage.install_root),
                "cache_dir": str(self.storage.cache_dir),
                "temp_dir": str(self.storage.temp_dir),
                "state_file": str(self.storage.state_file),
                "lock_file": str(self.storage.lock_file),
            },
            "cache": asdict(self.cache),
            "network": asdict(self.network),
            "security": asdict(self.security),
            "hooks": asdict(self.hooks),
        }
        return config_dict

    def save(self, config_path: Path) -> None:
        config_path.parent.mkdir(parents=True, exist_ok=True)

        with open(config_path, "w") as f:
            json.dump(self.to_dict(), f, indent=2)

    @classmethod
    def default(cls) -> "CocoatlyConfig":
        home_dir = Path.home()
        cocoatly_home = home_dir / ".cocoatly"

        registries = {
            "cocoatly-registry": RegistryEndpoint(
                url="https://registry.cocoatly.io",
                api_version="v1",
                requires_auth=False,
            )
        }

        registry = RegistryConfig(
            default_registry="cocoatly-registry",
            registries=registries,
            auth_tokens={},
        )

        storage = StorageConfig(
            install_root=cocoatly_home / "packages",
            cache_dir=cocoatly_home / "cache",
            temp_dir=cocoatly_home / "tmp",
            state_file=cocoatly_home / "state.json",
            lock_file=cocoatly_home / "cocoatly.lock",
        )

        cache = CacheConfig()
        network = NetworkConfig()
        security = SecurityConfig()
        hooks = HooksConfig()

        return cls(
            registry=registry,
            storage=storage,
            cache=cache,
            network=network,
            security=security,
            hooks=hooks,
        )

    @staticmethod
    def default_config_path() -> Path:
        home_dir = Path.home()
        return home_dir / ".cocoatly" / "config.json"

    def ensure_directories(self) -> None:
        self.storage.install_root.mkdir(parents=True, exist_ok=True)
        self.storage.cache_dir.mkdir(parents=True, exist_ok=True)
        self.storage.temp_dir.mkdir(parents=True, exist_ok=True)

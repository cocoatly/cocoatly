from dataclasses import dataclass, field
from typing import List, Optional, Dict
from datetime import datetime
from enum import Enum

@dataclass
class Version:
    major: int
    minor: int
    patch: int
    prerelease: Optional[str] = None
    build: Optional[str] = None

    def __str__(self) -> str:
        version = f"{self.major}.{self.minor}.{self.patch}"
        if self.prerelease:
            version += f"-{self.prerelease}"
        if self.build:
            version += f"+{self.build}"
        return version

    @classmethod
    def parse(cls, version_str: str) -> "Version":
        parts = version_str.split(".")
        if len(parts) < 3:
            raise ValueError(f"Invalid version string: {version_str}")

        major = int(parts[0])
        minor = int(parts[1])
        patch_parts = parts[2].split("-")
        patch = int(patch_parts[0].split("+")[0])

        prerelease = None
        build = None

        if "-" in parts[2]:
            prerelease = parts[2].split("-")[1].split("+")[0]

        if "+" in parts[2]:
            build = parts[2].split("+")[1]

        return cls(major, minor, patch, prerelease, build)

    def __lt__(self, other: "Version") -> bool:
        if self.major != other.major:
            return self.major < other.major
        if self.minor != other.minor:
            return self.minor < other.minor
        return self.patch < other.patch

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Version):
            return False
        return (self.major == other.major and
                self.minor == other.minor and
                self.patch == other.patch)

    def __le__(self, other: "Version") -> bool:
        return self < other or self == other

    def __gt__(self, other: "Version") -> bool:
        return not self <= other

    def __ge__(self, other: "Version") -> bool:
        return not self < other

class VersionRequirement:
    def __init__(self, requirement_str: str):
        self.requirement_str = requirement_str
        self.operator, self.version = self._parse(requirement_str)

    def _parse(self, req: str):
        req = req.strip()

        if req == "*":
            return ("any", None)

        if req.startswith("^"):
            return ("compatible", Version.parse(req[1:]))
        elif req.startswith(">="):
            return ("gte", Version.parse(req[2:].strip()))
        elif req.startswith(">"):
            return ("gt", Version.parse(req[1:].strip()))
        elif req.startswith("<="):
            return ("lte", Version.parse(req[2:].strip()))
        elif req.startswith("<"):
            return ("lt", Version.parse(req[1:].strip()))
        else:
            return ("exact", Version.parse(req))

    def matches(self, version: Version) -> bool:
        if self.operator == "any":
            return True
        elif self.operator == "exact":
            return version == self.version
        elif self.operator == "gt":
            return version > self.version
        elif self.operator == "gte":
            return version >= self.version
        elif self.operator == "lt":
            return version < self.version
        elif self.operator == "lte":
            return version <= self.version
        elif self.operator == "compatible":
            if version < self.version:
                return False
            if self.version.major == 0:
                return version.major == 0 and version.minor == self.version.minor
            return version.major == self.version.major
        return False

@dataclass
class Dependency:
    name: str
    version_requirement: VersionRequirement
    optional: bool = False
    features: List[str] = field(default_factory=list)

@dataclass
class PackageMetadata:
    id: str
    name: str
    version: Version
    description: Optional[str] = None
    authors: List[str] = field(default_factory=list)
    license: Optional[str] = None
    homepage: Optional[str] = None
    repository: Optional[str] = None
    keywords: List[str] = field(default_factory=list)
    categories: List[str] = field(default_factory=list)
    created_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None

@dataclass
class Package:
    metadata: PackageMetadata
    dependencies: List[Dependency] = field(default_factory=list)
    dev_dependencies: List[Dependency] = field(default_factory=list)
    build_dependencies: List[Dependency] = field(default_factory=list)
    scripts: Dict[str, str] = field(default_factory=dict)
    features: Dict[str, List[str]] = field(default_factory=dict)

@dataclass
class PackageVersion:
    package_name: str
    version: Version
    download_url: str
    checksum: str
    checksum_algorithm: str
    signature: Optional[str] = None
    size: int = 0

@dataclass
class InstalledPackage:
    name: str
    version: Version
    install_path: str
    installed_at: datetime
    requested_by: List[str] = field(default_factory=list)
    files: List[str] = field(default_factory=list)

class OperationStatus(Enum):
    PENDING = "pending"
    DOWNLOADING = "downloading"
    VERIFYING = "verifying"
    EXTRACTING = "extracting"
    INSTALLING = "installing"
    COMPLETED = "completed"
    FAILED = "failed"
    CANCELLED = "cancelled"

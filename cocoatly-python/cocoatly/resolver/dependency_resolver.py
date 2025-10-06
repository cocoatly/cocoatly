from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass, field
from ..core.models import Package, Dependency, Version, VersionRequirement, PackageVersion
from ..core.exceptions import DependencyResolutionException
from ..registry.client import RegistryClient

@dataclass
class ResolvedDependency:
    name: str
    version: Version
    dependencies: List[Dependency] = field(default_factory=list)

@dataclass
class ResolutionPlan:
    packages: List[ResolvedDependency]
    install_order: List[str]

class DependencyResolver:
    def __init__(self, registry_client: RegistryClient):
        self.registry = registry_client
        self.resolved: Dict[str, ResolvedDependency] = {}
        self.resolving: Set[str] = set()

    def resolve(
        self,
        root_dependencies: List[Dependency],
        existing_packages: Optional[Dict[str, Version]] = None,
    ) -> ResolutionPlan:
        self.resolved = {}
        self.resolving = set()

        if existing_packages is None:
            existing_packages = {}

        for dep in root_dependencies:
            if dep.optional:
                continue

            self._resolve_dependency(dep, existing_packages)

        install_order = self._compute_install_order()

        return ResolutionPlan(
            packages=list(self.resolved.values()),
            install_order=install_order,
        )

    def _resolve_dependency(
        self,
        dependency: Dependency,
        existing_packages: Dict[str, Version],
    ) -> None:
        if dependency.name in self.resolved:
            existing_version = self.resolved[dependency.name].version
            if not dependency.version_requirement.matches(existing_version):
                raise DependencyResolutionException(
                    f"Version conflict for {dependency.name}: "
                    f"need {dependency.version_requirement.requirement_str}, "
                    f"already resolved to {existing_version}"
                )
            return

        if dependency.name in self.resolving:
            raise DependencyResolutionException(
                f"Circular dependency detected: {dependency.name}"
            )

        self.resolving.add(dependency.name)

        try:
            if dependency.name in existing_packages:
                existing_version = existing_packages[dependency.name]
                if dependency.version_requirement.matches(existing_version):
                    package = self.registry.get_package(dependency.name)
                    self.resolved[dependency.name] = ResolvedDependency(
                        name=dependency.name,
                        version=existing_version,
                        dependencies=package.dependencies,
                    )
                    return

            selected_version = self._select_version(dependency)

            package = self.registry.get_package(dependency.name)

            self.resolved[dependency.name] = ResolvedDependency(
                name=dependency.name,
                version=selected_version,
                dependencies=package.dependencies,
            )

            for sub_dep in package.dependencies:
                if sub_dep.optional:
                    continue
                self._resolve_dependency(sub_dep, existing_packages)

        finally:
            self.resolving.remove(dependency.name)

    def _select_version(self, dependency: Dependency) -> Version:
        versions = self.registry.get_package_versions(dependency.name)

        if not versions:
            raise DependencyResolutionException(
                f"No versions found for package {dependency.name}"
            )

        matching_versions = [
            v for v in versions
            if dependency.version_requirement.matches(v.version)
        ]

        if not matching_versions:
            raise DependencyResolutionException(
                f"No version of {dependency.name} matches requirement "
                f"{dependency.version_requirement.requirement_str}"
            )

        matching_versions.sort(key=lambda v: v.version, reverse=True)

        return matching_versions[0].version

    def _compute_install_order(self) -> List[str]:
        graph = self._build_dependency_graph()
        visited = set()
        order = []

        def visit(name: str):
            if name in visited:
                return
            visited.add(name)

            if name in graph:
                for dep_name in graph[name]:
                    visit(dep_name)

            order.append(name)

        for name in self.resolved.keys():
            visit(name)

        return order

    def _build_dependency_graph(self) -> Dict[str, List[str]]:
        graph = {}

        for name, resolved in self.resolved.items():
            graph[name] = [
                dep.name
                for dep in resolved.dependencies
                if dep.name in self.resolved and not dep.optional
            ]

        return graph

def resolve_dependencies(
    registry_client: RegistryClient,
    root_dependencies: List[Dependency],
    existing_packages: Optional[Dict[str, Version]] = None,
) -> ResolutionPlan:
    resolver = DependencyResolver(registry_client)
    return resolver.resolve(root_dependencies, existing_packages)

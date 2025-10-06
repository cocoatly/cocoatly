import click
import json
from pathlib import Path
from rich.console import Console
from rich.table import Table
from rich.progress import Progress, SpinnerColumn, TextColumn
from ..core.config import CocoatlyConfig
from ..core.exceptions import CocoatlyException
from ..registry.client import RegistryClient
from ..bridge.rust_bridge import RustBridge
from ..resolver.dependency_resolver import DependencyResolver
from ..plugins.plugin_manager import PluginManager
from ..core.models import Dependency, VersionRequirement, Version

console = Console()

@click.group()
@click.option("--config", type=click.Path(exists=False), help="Path to config file")
@click.pass_context
def cli(ctx, config):
    ctx.ensure_object(dict)

    if config:
        config_path = Path(config)
    else:
        config_path = CocoatlyConfig.default_config_path()

    ctx.obj["config"] = CocoatlyConfig.load(config_path)
    ctx.obj["config"].ensure_directories()
    ctx.obj["registry"] = RegistryClient(ctx.obj["config"])
    ctx.obj["bridge"] = RustBridge(ctx.obj["config"])
    ctx.obj["plugin_manager"] = PluginManager()
    ctx.obj["plugin_manager"].load_plugins()

@cli.command()
@click.argument("package")
@click.option("--version", help="Specific version to install")
@click.pass_context
def install(ctx, package, version):
    config = ctx.obj["config"]
    registry = ctx.obj["registry"]
    bridge = ctx.obj["bridge"]
    plugin_manager = ctx.obj["plugin_manager"]

    try:
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task(f"Resolving dependencies for {package}...", total=None)

            if version:
                package_version = registry.get_package_version(package, version)
            else:
                versions = registry.get_package_versions(package)
                if not versions:
                    console.print(f"[red]No versions found for package {package}[/red]")
                    return
                package_version = versions[0]

            progress.update(task, description=f"Fetching package metadata for {package}...")
            package_data = registry.get_package(package)

            progress.update(task, description="Resolving dependencies...")
            dependencies = [
                Dependency(
                    name=dep["name"],
                    version_requirement=VersionRequirement(dep["version_requirement"])
                )
                for dep in package_data.get("dependencies", [])
            ]

            resolver = DependencyResolver(registry)
            state = bridge.read_state()
            existing_packages = {
                pkg["name"]: Version.parse(f"{pkg['version']['major']}.{pkg['version']['minor']}.{pkg['version']['patch']}")
                for pkg in state.get("installed_packages", {}).values()
            }

            resolution_plan = resolver.resolve(dependencies, existing_packages)

            progress.update(task, description="Installing packages...")
            plugin_manager.trigger_hook("pre_install", package, str(package_version.version))

            artifact = {
                "package_id": package_data["id"],
                "name": package,
                "version": {
                    "major": package_version.version.major,
                    "minor": package_version.version.minor,
                    "patch": package_version.version.patch,
                },
                "download_url": package_version.download_url,
                "checksum": package_version.checksum,
                "checksum_algorithm": package_version.checksum_algorithm,
                "signature": package_version.signature,
                "size": package_version.size,
            }

            for pkg_name in resolution_plan.install_order:
                if pkg_name not in existing_packages:
                    resolved = next(p for p in resolution_plan.packages if p.name == pkg_name)
                    progress.update(task, description=f"Installing {pkg_name} {resolved.version}...")

            result = bridge.install_package(artifact)

            plugin_manager.trigger_hook("post_install", package, str(package_version.version))

            progress.update(task, description="Done!", completed=True)

        console.print(f"[green]Successfully installed {package} {package_version.version}[/green]")

    except CocoatlyException as e:
        console.print(f"[red]Error: {str(e)}[/red]")
    except Exception as e:
        console.print(f"[red]Unexpected error: {str(e)}[/red]")

@cli.command()
@click.argument("package")
@click.pass_context
def uninstall(ctx, package):
    bridge = ctx.obj["bridge"]
    plugin_manager = ctx.obj["plugin_manager"]

    try:
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task(f"Uninstalling {package}...", total=None)

            state = bridge.read_state()
            installed_packages = state.get("installed_packages", {})

            if package not in installed_packages:
                console.print(f"[yellow]Package {package} is not installed[/yellow]")
                return

            pkg_info = installed_packages[package]
            version = f"{pkg_info['version']['major']}.{pkg_info['version']['minor']}.{pkg_info['version']['patch']}"

            plugin_manager.trigger_hook("pre_uninstall", package, version)

            result = bridge.uninstall_package(package)

            plugin_manager.trigger_hook("post_uninstall", package, version)

            progress.update(task, description="Done!", completed=True)

        console.print(f"[green]Successfully uninstalled {package}[/green]")

    except CocoatlyException as e:
        console.print(f"[red]Error: {str(e)}[/red]")
    except Exception as e:
        console.print(f"[red]Unexpected error: {str(e)}[/red]")

@cli.command()
@click.argument("query")
@click.option("--limit", default=20, help="Number of results to show")
@click.pass_context
def search(ctx, query, limit):
    registry = ctx.obj["registry"]

    try:
        results = registry.search_packages(query, limit=limit)

        if not results["packages"]:
            console.print(f"[yellow]No packages found matching '{query}'[/yellow]")
            return

        table = Table(title=f"Search Results for '{query}'")
        table.add_column("Name", style="cyan")
        table.add_column("Description", style="white")
        table.add_column("Downloads", style="green")

        for package in results["packages"]:
            table.add_row(
                package["name"],
                package.get("description", "")[:60],
                str(package.get("downloads_total", 0)),
            )

        console.print(table)

    except CocoatlyException as e:
        console.print(f"[red]Error: {str(e)}[/red]")

@cli.command()
@click.pass_context
def list(ctx):
    bridge = ctx.obj["bridge"]

    try:
        state = bridge.read_state()
        installed_packages = state.get("installed_packages", {})

        if not installed_packages:
            console.print("[yellow]No packages installed[/yellow]")
            return

        table = Table(title="Installed Packages")
        table.add_column("Name", style="cyan")
        table.add_column("Version", style="green")
        table.add_column("Install Path", style="white")

        for name, pkg_info in installed_packages.items():
            version = f"{pkg_info['version']['major']}.{pkg_info['version']['minor']}.{pkg_info['version']['patch']}"
            table.add_row(name, version, pkg_info.get("install_path", ""))

        console.print(table)

    except Exception as e:
        console.print(f"[red]Error: {str(e)}[/red]")

@cli.command()
@click.argument("package")
@click.pass_context
def info(ctx, package):
    registry = ctx.obj["registry"]

    try:
        package_data = registry.get_package(package)

        console.print(f"[bold cyan]{package_data['name']}[/bold cyan]")
        console.print(f"Description: {package_data.get('description', 'N/A')}")
        console.print(f"License: {package_data.get('license', 'N/A')}")
        console.print(f"Homepage: {package_data.get('homepage', 'N/A')}")
        console.print(f"Repository: {package_data.get('repository', 'N/A')}")
        console.print(f"Total Downloads: {package_data.get('downloads_total', 0)}")

        if package_data.get("keywords"):
            console.print(f"Keywords: {', '.join(package_data['keywords'])}")

        if package_data.get("authors"):
            console.print(f"Authors: {', '.join(package_data['authors'])}")

    except CocoatlyException as e:
        console.print(f"[red]Error: {str(e)}[/red]")

@cli.command()
@click.argument("package")
@click.pass_context
def verify(ctx, package):
    bridge = ctx.obj["bridge"]

    try:
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task(f"Verifying {package}...", total=None)

            result = bridge.verify_package(package)

            progress.update(task, description="Done!", completed=True)

        if result.get("valid"):
            console.print(f"[green]Package {package} is valid[/green]")
        else:
            console.print(f"[red]Package {package} verification failed[/red]")
            if result.get("missing_files"):
                console.print(f"Missing files: {len(result['missing_files'])}")
            if result.get("corrupted_files"):
                console.print(f"Corrupted files: {len(result['corrupted_files'])}")

    except Exception as e:
        console.print(f"[red]Error: {str(e)}[/red]")

@cli.command()
@click.pass_context
def plugins(ctx):
    plugin_manager = ctx.obj["plugin_manager"]

    plugins_list = plugin_manager.list_plugins()

    if not plugins_list:
        console.print("[yellow]No plugins loaded[/yellow]")
        return

    table = Table(title="Loaded Plugins")
    table.add_column("Name", style="cyan")
    table.add_column("Version", style="green")
    table.add_column("Description", style="white")
    table.add_column("Author", style="magenta")

    for plugin_meta in plugins_list:
        table.add_row(
            plugin_meta.name,
            plugin_meta.version,
            plugin_meta.description,
            plugin_meta.author,
        )

    console.print(table)

if __name__ == "__main__":
    cli()

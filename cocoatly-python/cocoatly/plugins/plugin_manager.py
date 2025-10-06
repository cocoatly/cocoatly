import importlib
import sys
from pathlib import Path
from typing import Dict, List, Any, Callable, Optional
from abc import ABC, abstractmethod
from dataclasses import dataclass

@dataclass
class PluginMetadata:
    name: str
    version: str
    description: str
    author: str

class Plugin(ABC):
    @abstractmethod
    def get_metadata(self) -> PluginMetadata:
        pass

    def on_pre_install(self, package_name: str, version: str) -> None:
        pass

    def on_post_install(self, package_name: str, version: str) -> None:
        pass

    def on_pre_uninstall(self, package_name: str, version: str) -> None:
        pass

    def on_post_uninstall(self, package_name: str, version: str) -> None:
        pass

    def on_package_update(self, package_name: str, old_version: str, new_version: str) -> None:
        pass

    def add_cli_commands(self) -> Dict[str, Callable]:
        return {}

class PluginManager:
    def __init__(self, plugin_dir: Optional[Path] = None):
        if plugin_dir is None:
            plugin_dir = Path.home() / ".cocoatly" / "plugins"

        self.plugin_dir = plugin_dir
        self.plugins: Dict[str, Plugin] = {}
        self.hooks: Dict[str, List[Callable]] = {
            "pre_install": [],
            "post_install": [],
            "pre_uninstall": [],
            "post_uninstall": [],
            "package_update": [],
        }

    def load_plugins(self) -> None:
        if not self.plugin_dir.exists():
            return

        sys.path.insert(0, str(self.plugin_dir))

        for plugin_path in self.plugin_dir.glob("*.py"):
            if plugin_path.stem.startswith("_"):
                continue

            try:
                module_name = plugin_path.stem
                module = importlib.import_module(module_name)

                for attr_name in dir(module):
                    attr = getattr(module, attr_name)

                    if isinstance(attr, type) and issubclass(attr, Plugin) and attr is not Plugin:
                        plugin_instance = attr()
                        metadata = plugin_instance.get_metadata()
                        self.plugins[metadata.name] = plugin_instance

                        self._register_hooks(plugin_instance)

            except Exception as e:
                print(f"Failed to load plugin {plugin_path}: {e}")

    def _register_hooks(self, plugin: Plugin) -> None:
        self.hooks["pre_install"].append(plugin.on_pre_install)
        self.hooks["post_install"].append(plugin.on_post_install)
        self.hooks["pre_uninstall"].append(plugin.on_pre_uninstall)
        self.hooks["post_uninstall"].append(plugin.on_post_uninstall)
        self.hooks["package_update"].append(plugin.on_package_update)

    def trigger_hook(self, hook_name: str, *args, **kwargs) -> None:
        if hook_name not in self.hooks:
            return

        for hook_func in self.hooks[hook_name]:
            try:
                hook_func(*args, **kwargs)
            except Exception as e:
                print(f"Plugin hook '{hook_name}' failed: {e}")

    def get_plugin(self, plugin_name: str) -> Optional[Plugin]:
        return self.plugins.get(plugin_name)

    def list_plugins(self) -> List[PluginMetadata]:
        return [plugin.get_metadata() for plugin in self.plugins.values()]

    def get_cli_commands(self) -> Dict[str, Callable]:
        commands = {}

        for plugin in self.plugins.values():
            plugin_commands = plugin.add_cli_commands()
            commands.update(plugin_commands)

        return commands

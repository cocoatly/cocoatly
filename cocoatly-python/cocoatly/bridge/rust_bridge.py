import json
import subprocess
from pathlib import Path
from typing import Dict, Any, Optional
from ..core.exceptions import RustBridgeException
from ..core.config import CocoatlyConfig

class RustBridge:
    def __init__(self, config: CocoatlyConfig, rust_bin_dir: Optional[Path] = None):
        self.config = config

        if rust_bin_dir is None:
            rust_bin_dir = Path(__file__).parent.parent.parent.parent / "cocoatly-rust" / "target" / "release"

        self.rust_bin_dir = rust_bin_dir
        self.install_bin = self.rust_bin_dir / "cocoatly-install"
        self.uninstall_bin = self.rust_bin_dir / "cocoatly-uninstall"
        self.verify_bin = self.rust_bin_dir / "cocoatly-verify"
        self.state_bin = self.rust_bin_dir / "cocoatly-state"

    def _execute_rust_binary(
        self,
        binary_path: Path,
        args: list,
        operation_name: str,
    ) -> Dict[str, Any]:
        if not binary_path.exists():
            raise RustBridgeException(
                operation_name,
                f"Rust binary not found: {binary_path}. Please build the Rust components first."
            )

        try:
            result = subprocess.run(
                [str(binary_path)] + args,
                capture_output=True,
                text=True,
                check=False,
            )

            try:
                output = json.loads(result.stdout)
            except json.JSONDecodeError:
                raise RustBridgeException(
                    operation_name,
                    f"Failed to parse Rust output: {result.stdout}\nStderr: {result.stderr}"
                )

            if not output.get("success", False):
                error_msg = output.get("error", "Unknown error")
                raise RustBridgeException(operation_name, error_msg)

            return output.get("data", {})

        except subprocess.SubprocessError as e:
            raise RustBridgeException(operation_name, f"Subprocess error: {str(e)}")

    def install_package(self, artifact: Dict[str, Any]) -> Dict[str, Any]:
        config_path = self.config.storage.state_file.parent / "config.json"
        self.config.save(config_path)

        args = [
            "--config", str(config_path),
            "--artifact-json", json.dumps(artifact),
        ]

        return self._execute_rust_binary(
            self.install_bin,
            args,
            "install"
        )

    def uninstall_package(self, package_name: str) -> Dict[str, Any]:
        config_path = self.config.storage.state_file.parent / "config.json"
        self.config.save(config_path)

        args = [
            "--config", str(config_path),
            "--package", package_name,
        ]

        return self._execute_rust_binary(
            self.uninstall_bin,
            args,
            "uninstall"
        )

    def verify_package(self, package_name: str) -> Dict[str, Any]:
        config_path = self.config.storage.state_file.parent / "config.json"
        self.config.save(config_path)

        args = [
            "--config", str(config_path),
            "--package", package_name,
        ]

        return self._execute_rust_binary(
            self.verify_bin,
            args,
            "verify"
        )

    def read_state(self) -> Dict[str, Any]:
        config_path = self.config.storage.state_file.parent / "config.json"
        self.config.save(config_path)

        args = [
            "read",
            "--config", str(config_path),
        ]

        result = subprocess.run(
            [str(self.state_bin)] + args,
            capture_output=True,
            text=True,
            check=False,
        )

        if result.returncode != 0:
            raise RustBridgeException("read_state", result.stderr)

        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            raise RustBridgeException("read_state", f"Failed to parse state: {result.stdout}")

    def write_state(self, state: Dict[str, Any]) -> None:
        config_path = self.config.storage.state_file.parent / "config.json"
        self.config.save(config_path)

        args = [
            "write",
            "--config", str(config_path),
            "--state-json", json.dumps(state),
        ]

        subprocess.run(
            [str(self.state_bin)] + args,
            capture_output=True,
            text=True,
            check=True,
        )

class CocoatlyException(Exception):
    pass

class PackageNotFoundException(CocoatlyException):
    def __init__(self, package_name: str):
        self.package_name = package_name
        super().__init__(f"Package not found: {package_name}")

class DependencyResolutionException(CocoatlyException):
    def __init__(self, message: str):
        super().__init__(f"Dependency resolution failed: {message}")

class InstallationException(CocoatlyException):
    def __init__(self, package_name: str, message: str):
        self.package_name = package_name
        super().__init__(f"Installation failed for {package_name}: {message}")

class ConfigurationException(CocoatlyException):
    def __init__(self, message: str):
        super().__init__(f"Configuration error: {message}")

class RegistryException(CocoatlyException):
    def __init__(self, message: str):
        super().__init__(f"Registry error: {message}")

class VerificationException(CocoatlyException):
    def __init__(self, message: str):
        super().__init__(f"Verification failed: {message}")

class RustBridgeException(CocoatlyException):
    def __init__(self, operation: str, message: str):
        self.operation = operation
        super().__init__(f"Rust operation '{operation}' failed: {message}")

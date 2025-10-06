# Cocoatly Package Manager Architecture

## Overview

Cocoatly is a production-grade package manager built with a hybrid architecture combining Rust for performance-critical operations and Python for high-level orchestration.

## Architecture Principles

### Rust Layer (Performance-Critical Operations)
**Location**: `cocoatly-rust/`

Handles all heavy lifting:
- **Package Installation/Uninstallation**: File system operations, extraction, installation
- **Cryptographic Operations**: Hash computation, signature verification
- **File System Management**: Directory operations, file copying, symlinks
- **Compression/Decompression**: tar.gz archive handling
- **Download Management**: Concurrent downloads with retry logic

#### Rust Crates Structure

```
cocoatly-rust/
├── crates/
│   ├── cocoatly-core/          # Core types, error handling, config
│   ├── cocoatly-crypto/        # Cryptography (blake3, sha256, signatures)
│   ├── cocoatly-compression/   # Archive operations (tar.gz)
│   ├── cocoatly-downloader/    # HTTP downloads with concurrency
│   ├── cocoatly-fs/            # File system operations
│   ├── cocoatly-installer/     # Installation orchestration
│   └── cocoatly-cli-bridge/    # CLI binaries for Python IPC
```

### Python Layer (High-Level Orchestration)
**Location**: `cocoatly-python/`

Manages intelligent operations:
- **CLI Interface**: User-facing command-line interface with rich output
- **Dependency Resolution**: Complex version constraint solving
- **Registry API Client**: Communication with package registry
- **Configuration Management**: User preferences and settings
- **Plugin System**: Extensible hook-based architecture

#### Python Package Structure

```
cocoatly-python/
├── cocoatly/
│   ├── core/           # Configuration, models, exceptions
│   ├── cli/            # Command-line interface
│   ├── registry/       # Registry API client
│   ├── resolver/       # Dependency resolution engine
│   ├── bridge/         # Rust IPC bridge
│   └── plugins/        # Plugin management system
```

## Inter-Process Communication (IPC)

### Communication Methods

1. **CLI Wrapper (Primary)**
   - Python invokes Rust binaries as subprocesses
   - JSON-based input/output for structured data exchange
   - Exit codes for operation status

2. **Shared State Files**
   - `~/.cocoatly/state.json` - Global installation state
   - `~/.cocoatly/config.json` - Configuration
   - `~/.cocoatly/cocoatly.lock` - Lock file for concurrent operations

### Rust Binaries

- `cocoatly-install` - Installs a package from artifact metadata
- `cocoatly-uninstall` - Removes an installed package
- `cocoatly-verify` - Verifies package integrity
- `cocoatly-state` - Read/write global state

All binaries accept JSON input and produce JSON output with standardized format:
```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## Data Flow

### Installation Flow

1. **Python CLI** receives `cocoatly install <package>` command
2. **Registry Client** queries Supabase for package metadata
3. **Dependency Resolver** computes full dependency tree
4. **Python** prepares artifact metadata (URL, checksums, signatures)
5. **Rust Bridge** invokes `cocoatly-install` binary with artifact JSON
6. **Rust Installer**:
   - Downloads package from registry
   - Verifies checksum and signature
   - Extracts archive to temporary directory
   - Moves files to installation directory
   - Updates global state
7. **Python** reads updated state and confirms installation

### Uninstallation Flow

1. **Python CLI** receives `cocoatly uninstall <package>` command
2. **Rust Bridge** invokes `cocoatly-uninstall` binary with package name
3. **Rust Uninstaller**:
   - Reads global state
   - Removes package files
   - Updates global state
4. **Python** confirms removal

## Database Schema (Supabase)

### Core Tables

- **packages** - Package metadata (name, description, license, downloads)
- **package_versions** - Version-specific data (semver, download URLs, checksums)
- **package_authors** - Author information
- **package_keywords** - Search keywords
- **package_categories** - Package categorization
- **dependencies** - Dependency relationships
- **download_stats** - Analytics and download tracking

### Security

All tables have Row Level Security (RLS) enabled:
- Public read access for package discovery
- Authenticated write access for publishing
- No anonymous write operations

## Dependency Resolution

The resolver implements a constraint satisfaction algorithm:

1. Parse version requirements (exact, range, compatible)
2. Build dependency graph from root package
3. Detect circular dependencies
4. Resolve version conflicts using constraint propagation
5. Compute topological installation order

Version requirement formats:
- `1.2.3` - Exact version
- `^1.2.3` - Compatible version (1.x.x, x >= 2)
- `>=1.2.3` - Greater than or equal
- `>1.2.3` - Greater than
- `<=1.2.3` - Less than or equal
- `<1.2.3` - Less than
- `*` - Any version

## Plugin System

Plugins extend functionality through lifecycle hooks:

- `on_pre_install` - Before package installation
- `on_post_install` - After package installation
- `on_pre_uninstall` - Before package removal
- `on_post_uninstall` - After package removal
- `on_package_update` - During package updates
- `add_cli_commands` - Add custom CLI commands

Plugins are Python modules placed in `~/.cocoatly/plugins/`.

## Configuration

Configuration file: `~/.cocoatly/config.json`

Sections:
- **registry** - Registry endpoints and authentication
- **storage** - Installation paths and state files
- **cache** - Cache settings and cleanup policies
- **network** - Timeouts, retries, proxy configuration
- **security** - Signature/checksum verification settings
- **hooks** - Shell commands for lifecycle events

## Performance Characteristics

- **Concurrent Downloads**: Up to 8 simultaneous package downloads
- **Cryptographic Verification**: BLAKE3 (fastest), SHA256, SHA512 support
- **Compression**: Gzip with best compression ratio
- **State Management**: Atomic writes with lock files

## Security Features

1. **Cryptographic Verification**
   - BLAKE3/SHA256/SHA512 checksums for all packages
   - Ed25519 digital signatures for publisher verification

2. **Row Level Security**
   - Database-level access control
   - Authenticated writes only

3. **Sandboxed Execution**
   - Rust's memory safety guarantees
   - No arbitrary code execution during installation

4. **Audit Trail**
   - Download statistics with IP and user agent
   - Package modification timestamps

## Scalability

- **Registry**: Supabase with PostgreSQL backend (horizontal scaling)
- **Downloads**: CDN-friendly with URL-based artifacts
- **Local Storage**: Efficient filesystem hierarchy by package name and version
- **Concurrent Operations**: Lock file prevents state corruption

## Future Extensions

- Binary cache for pre-compiled packages
- Private registry support
- Workspace/monorepo support
- Build system integration
- Container image generation

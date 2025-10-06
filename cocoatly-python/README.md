# Cocoatly Package Manager

A high-performance, production-grade package manager built with Rust and Python.

## Architecture

- **Rust**: Handles performance-critical operations (installation, file system operations, cryptography, compression, downloads)
- **Python**: Manages high-level orchestration (CLI interface, dependency resolution, registry API, configuration)

## Installation

```bash
pip install -e .
```

## Usage

```bash
cocoatly install <package>
cocoatly uninstall <package>
cocoatly search <query>
cocoatly list
cocoatly update <package>
```

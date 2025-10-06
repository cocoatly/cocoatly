# Building Cocoatly Rust Components

## Prerequisites

Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build Instructions

Build all Rust components:
```bash
cd cocoatly-rust
cargo build --release
```

The compiled binaries will be located in:
```
cocoatly-rust/target/release/
```

Binaries include:
- `cocoatly-install` - Package installation binary
- `cocoatly-uninstall` - Package uninstallation binary
- `cocoatly-verify` - Package verification binary
- `cocoatly-state` - State management binary

## Python Setup

After building Rust components, install Python package:
```bash
cd cocoatly-python
pip install -e .
```

## Usage

```bash
cocoatly install <package>
cocoatly uninstall <package>
cocoatly search <query>
cocoatly list
cocoatly info <package>
cocoatly verify <package>
```

# Tunic Engine

The core cryptographic and vanity address generation engine for Tunic Wallet. This repository contains high-performance Rust implementations for EVM and Solana, designed for both WASM (Web) and Native (CLI/Server) environments.

## Repository Overview

This engine is structured as a collection of specialized crates:

### 1. [evm-engine](./evm-engine)
Core Rust logic for Ethereum and EVM-compatible vanity address generation.
- **Features**: secp256k1 key derivation, Keccak-256 hashing.
- **Targets**: WASM (via `wasm-bindgen`) and Standard.

### 2. [solana-engine](./solana-engine)
Core Rust logic for Solana vanity address generation.
- **Features**: ed25519 key derivation, Base58 encoding.
- **Targets**: WASM and Standard.

### 3. [tunic-cli](./tunic-cli)
The professional command-line interface for the search engine.
- **Features**: Industrial-grade brute-forcing using **`rayon`** for multi-threaded parallel processing.
- **Interface**: Subcommand-based (`tunic generate`) with support for prefix, suffix, and combined matching.

## Installation

### Local Build & Install
Use the root `Makefile` for a seamless installation:

```bash
# Build and install the 'tunic' CLI globally
make install
```

### Manual Install
```bash
cargo install --path tunic-cli
```

## Professional Automation
- **GitHub Actions**: Automated binary releases for macOS, Linux, and Windows are triggered on every version tag (`v*`).
- **Performance**: Capable of generating millions of addresses per second across all CPU cores.

---

*For detailed instructions on each component, please refer to the README.md inside their respective folders.*

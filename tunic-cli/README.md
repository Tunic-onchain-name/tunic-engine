# Tunic CLI

High-performance EVM vanity address generator.

## Distribution

### 1. Download Pre-built Binaries (Recommended)
You can download the latest pre-built binaries for macOS, Linux, and Windows from the [GitHub Releases](https://github.com/Tunic-onchain-name/tunic-engine/releases) page.

### 2. Install from Source (via Makefile)
If you have the repository cloned and the [Rust](https://rustup.rs/) toolchain installed:
```bash
make install
```
This will build and install the `tunic` binary to your `~/.cargo/bin` directory.

### 3. Direct Cargo Install
```bash
cargo install --path tunic-cli
```
Or directly from GitHub:
```bash
cargo install --git https://github.com/Tunic-onchain-name/tunic-engine --path tunic-cli
```

## Usage

### Generate with Prefix
```bash
tunic generate --mode prefix --prefix dead
```

### Generate with Suffix
```bash
tunic generate --mode suffix --suffix beef
```

### Generate with Combine
```bash
tunic generate --mode combine --combine de:ad
```

## Flags
- `-m, --mode <MODE>`: prefix, suffix, or combine
- `--prefix <HEX>`: 0x-prefixed or raw hex
- `--suffix <HEX>`: raw hex
- `--combine <PREFIX:SUFFIX>`: separated by colon

## Performance
Tunic uses the `rayon` crate to leverage all available CPU cores for industrial-grade brute-forcing speed.
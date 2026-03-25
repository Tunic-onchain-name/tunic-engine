# Tunic CLI

High-performance, industrial-grade vanity address generator for EVM and beyond. Built in Rust for maximum speed and security.

## 🚀 Installation

For the best experience and to avoid OS security blocks (macOS Gatekeeper/Windows SmartScreen), we recommend installing via **Cargo**. This builds the binary locally on your machine, ensuring it is fully trusted by your system.

### Option 1: Cargo (Recommended & Trusted)
This is the most secure way to install `tunic`. It compiles the source code locally.

```bash
# For Public Repository (HTTPS)
cargo install --git https://github.com/Tunic-onchain-name/tunic-engine tunic-cli

# For Private Repository (SSH)
cargo install --git ssh://git@github.com/Tunic-onchain-name/tunic-engine.git tunic-cli
```

### Option 2: Pre-built Binaries
If you don't have Rust installed, you can download binaries for macOS, Linux, and Windows:
👉 [Latest Releases](https://github.com/Tunic-onchain-name/tunic-engine/releases)

> [!IMPORTANT]
> **macOS Users**: If you see a "developer cannot be verified" warning, right-click the binary in Finder and select **Open**, or run `xattr -d com.apple.quarantine <path-to-binary>`.

### Option 3: Build from Source
```bash
git clone https://github.com/Tunic-onchain-name/tunic-engine.git
cd tunic-engine
make install
```

---

## 🛠 Usage

The `tunic` CLI supports multiple matching modes to find your perfect vanity address.

### 1. Prefix Matching
Finds an address starting with your chosen characters.
```bash
tunic generate --mode prefix --prefix dead
```

### 2. Suffix Matching
Finds an address ending with your chosen characters.
```bash
tunic generate --mode suffix --suffix beef
```

### 3. Combined Matching
Finds an address that has both a specific prefix and suffix.
```bash
tunic generate --mode combine --combine de:ad
```

---

## 🚩 Commands & Flags

| Flag | Description |
|---|---|
| `-m, --mode <MODE>` | Selection mode: `prefix`, `suffix`, or `combine` |
| `--prefix <HEX>` | The prefix pattern to match (e.g., `dead`) |
| `--suffix <HEX>` | The suffix pattern to match (e.g., `beef`) |
| `--combine <P:S>` | Combined pattern separated by colon (e.g., `abc:123`) |

## ⚡ Performance
Tunic is built for speed. It leverages **Rayon** to distribute work across all available CPU cores, allowing you to brute-force millions of addresses per second.

---

## 🔒 Security
- **100% Offline**: Tunic never makes network requests. Your private keys never leave your machine.
- **Open Source**: Auditable Rust code using standard cryptographic primitives (`k256`, `tiny-keccak`).
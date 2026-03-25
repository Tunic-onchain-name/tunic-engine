.PHONY: build install clean help

# Metadata
BINARY_NAME=tunic
INSTALL_DIR=$(HOME)/.cargo/bin

help:
	@echo "Tunic Wallet Search Engine"
	@echo "--------------------------------"
	@echo "Usage:"
	@echo "  make build    - Build the 'tunic' CLI binary in release mode"
	@echo "  make install  - Install 'tunic' CLI globally to $(INSTALL_DIR)"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make help     - Show this help message"

build:
	@echo "Building $(BINARY_NAME) CLI..."
	cargo build --release --package tunic-cli

install:
	@echo "Installing $(BINARY_NAME) CLI..."
	cargo install --path tunic-cli
	@echo "Done. Ensure $(INSTALL_DIR) is in your PATH."

clean:
	@echo "Cleaning artifacts..."
	cargo clean

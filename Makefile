.PHONY: all build dev release install clean ui ui-dev lint test help

# Variables
BINARY_NAME := sysmon
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
BUILD_DIR := target/release
UI_DIR := ui
UI_BUILD_DIR := $(UI_DIR)/build
INSTALL_DIR := /usr/local/bin

# Default target
all: build

# Build UI first, then Rust binary
build: ui
	@echo "🦀 Building sysmon..."
	cargo build --release
	@echo "✅ Binary: $(BUILD_DIR)/$(BINARY_NAME) ($$(du -h $(BUILD_DIR)/$(BINARY_NAME) | cut -f1))"

# Development mode - run UI dev server + cargo watch
dev:
	@echo "🔧 Starting development mode..."
	@echo "Run these in separate terminals:"
	@echo "  Terminal 1: cd ui && npm run dev"
	@echo "  Terminal 2: cargo watch -x run"

# Build UI
ui:
	@echo "⚡ Building Svelte UI..."
	@cd $(UI_DIR) && npm install --silent && npm run build
	@echo "✅ UI built to $(UI_BUILD_DIR)/"

# Build UI dev server
ui-dev:
	@cd $(UI_DIR) && npm run dev

# Build release binary (optimized)
release: ui
	@echo "🚀 Building release binary..."
	cargo build --release
	@strip $(BUILD_DIR)/$(BINARY_NAME) 2>/dev/null || true
	@echo "✅ Release binary: $(BUILD_DIR)/$(BINARY_NAME) ($$(du -h $(BUILD_DIR)/$(BINARY_NAME) | cut -f1))"
	@echo "   Version: $(VERSION)"

# Build universal macOS binary (arm64 + x86_64)
universal: ui
	@echo "🍎 Building universal macOS binary..."
	cargo build --release --target aarch64-apple-darwin
	cargo build --release --target x86_64-apple-darwin
	@mkdir -p $(BUILD_DIR)
	lipo -create \
		target/aarch64-apple-darwin/release/$(BINARY_NAME) \
		target/x86_64-apple-darwin/release/$(BINARY_NAME) \
		-output $(BUILD_DIR)/$(BINARY_NAME)-universal
	@echo "✅ Universal binary: $(BUILD_DIR)/$(BINARY_NAME)-universal"

# Install to /usr/local/bin
install: release
	@echo "📦 Installing sysmon to $(INSTALL_DIR)..."
	@sudo cp $(BUILD_DIR)/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	@sudo chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Installed! Run 'sysmon' to start."

# Uninstall
uninstall:
	@echo "🗑  Uninstalling sysmon..."
	@sudo rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Uninstalled."

# Run linter
lint:
	@echo "🔍 Running lints..."
	cargo clippy -- -D warnings
	@cd $(UI_DIR) && npm run check 2>/dev/null || true

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test
	@cd $(UI_DIR) && npm run test

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean
	@rm -rf $(UI_BUILD_DIR)
	@rm -rf $(UI_DIR)/node_modules/.vite
	@echo "✅ Clean."

# Show version
version:
	@echo "sysmon v$(VERSION)"

# Help
help:
	@echo "sysmon - Lightweight System Monitor"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build      Build the binary (default)"
	@echo "  dev        Start development mode"
	@echo "  ui         Build Svelte UI only"
	@echo "  release    Build optimized release binary"
	@echo "  universal  Build universal macOS binary (arm64 + x86_64)"
	@echo "  install    Install to /usr/local/bin"
	@echo "  uninstall  Remove from /usr/local/bin"
	@echo "  lint       Run clippy and svelte-check"
	@echo "  test       Run tests"
	@echo "  clean      Remove build artifacts"
	@echo "  version    Show version"
	@echo "  help       Show this help"

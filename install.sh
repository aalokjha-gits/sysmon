#!/bin/sh
# sysmon installer
# Usage: curl -sSL https://raw.githubusercontent.com/user/sysmon/main/install.sh | sh
# Or: curl -sSL ... | sh -s -- --version v0.1.0

set -e

# Configuration
REPO="aalokjha-gits/sysmon"
BINARY_NAME="sysmon"
INSTALL_DIR="/usr/local/bin"

# Parse arguments
VERSION=""
while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --version=*)
            VERSION="${1#*=}"
            shift
            ;;
        -h|--help)
            echo "Usage: install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version <version>  Install specific version (default: latest)"
            echo "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors - POSIX-compliant (no \e)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print helpers
info() {
    printf "${BLUE}ℹ${NC}  %s\n" "$1"
}

success() {
    printf "${GREEN}✓${NC}  %s\n" "$1"
}

warn() {
    printf "${YELLOW}⚠${NC}  %s\n" "$1"
}

error() {
    printf "${RED}✗${NC}  %s\n" "$1" >&2
}

# Banner
cat << 'EOF'
┌─────────────────────────────────────────┐
│                                         │
│   ███████╗██╗   ██╗███████╗███╗   ███╗  │
│   ██╔════╝╚██╗ ██╔╝██╔════╝████╗ ████║  │
│   ███████╗ ╚████╔╝ ███████╗██╔████╔██║  │
│   ╚════██║  ╚██╔╝  ╚════██║██║╚██╔╝██║  │
│   ███████║   ██║   ███████║██║ ╚═╝ ██║  │
│   ╚══════╝   ╚═╝   ╚══════╝╚═╝     ╚═╝  │
│                                         │
│   Lightweight System Monitor            │
│                                         │
└─────────────────────────────────────────┘

EOF

info "Starting installation..."

# Detect OS
OS=$(uname -s)
ARCH=$(uname -m)

info "Detected: $OS ($ARCH)"

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64|aarch64)
                TARGET_ARCH="aarch64-apple-darwin"
                ;;
            x86_64|amd64)
                TARGET_ARCH="x86_64-apple-darwin"
                ;;
            *)
                error "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    Linux)
        case "$ARCH" in
            aarch64|arm64)
                TARGET_ARCH="aarch64-unknown-linux-gnu"
                ;;
            x86_64|amd64)
                TARGET_ARCH="x86_64-unknown-linux-gnu"
                ;;
            *)
                error "Unsupported architecture: $ARCH"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        error "Windows is not yet supported. Please use WSL2 or build from source."
        exit 1
        ;;
    *)
        error "Unsupported operating system: $OS"
        exit 1
        ;;
esac

info "Target architecture: $TARGET_ARCH"

# Get latest version if not specified
if [ -z "$VERSION" ]; then
    info "Fetching latest version..."
    # Try to get version from GitHub API, fallback to default
    LATEST=$(curl -sL "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null | \
        grep '"tag_name":' | head -1 | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')

    if [ -z "$LATEST" ] || [ "$LATEST" = "null" ]; then
        warn "Could not fetch latest version, using default"
        VERSION="v0.1.0"
    else
        VERSION="$LATEST"
    fi
fi

# Remove 'v' prefix if present for URL
VERSION_NO_V=$(echo "$VERSION" | sed 's/^v//')

success "Installing sysmon $VERSION"

# Check for required tools
if ! command -v curl >/dev/null 2>&1; then
    error "curl is required but not installed"
    exit 1
fi

# Check if install directory exists and is writable
if [ ! -d "$INSTALL_DIR" ]; then
    info "Creating $INSTALL_DIR..."
    if ! sudo mkdir -p "$INSTALL_DIR" 2>/dev/null; then
        error "Failed to create $INSTALL_DIR. Try running with sudo."
        exit 1
    fi
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/sysmon-${VERSION_NO_V}-${TARGET_ARCH}.tar.gz"
ALT_URL="https://github.com/$REPO/releases/download/$VERSION/sysmon-${TARGET_ARCH}"
UNIVERSAL_URL="https://github.com/$REPO/releases/download/$VERSION/sysmon-universal"

info "Downloading sysmon..."

DOWNLOADED=false

if [ "$OS" = "Darwin" ]; then
    if curl -sL --fail "$UNIVERSAL_URL" -o "$TMP_DIR/sysmon" 2>/dev/null; then
        success "Downloaded universal binary"
        DOWNLOADED=true
    fi
fi

if [ "$DOWNLOADED" = "false" ]; then
    if curl -sL --fail "$ALT_URL" -o "$TMP_DIR/sysmon" 2>/dev/null; then
        success "Downloaded binary"
        DOWNLOADED=true
    elif curl -sL --fail "$DOWNLOAD_URL" -o "$TMP_DIR/sysmon.tar.gz" 2>/dev/null; then
        info "Extracting archive..."
        tar -xzf "$TMP_DIR/sysmon.tar.gz" -C "$TMP_DIR"
        mv "$TMP_DIR/sysmon" "$TMP_DIR/sysmon" 2>/dev/null || true
        success "Downloaded and extracted"
        DOWNLOADED=true
    fi
fi

if [ "$DOWNLOADED" = "false" ]; then
    error "Failed to download sysmon"
    error "URLs tried:"
    error "  - $ALT_URL"
    error "  - $DOWNLOAD_URL"
    if [ "$OS" = "Darwin" ]; then
        error "  - $UNIVERSAL_URL"
    fi
    error ""
    error "You may need to build from source:"
    error "  git clone https://github.com/$REPO"
    error "  cd sysmon && make install"
    exit 1
fi

# Make executable
chmod +x "$TMP_DIR/sysmon"

# Verify binary
info "Verifying binary..."
if ! "$TMP_DIR/sysmon" --version >/dev/null 2>&1; then
    error "Downloaded binary does not work on this system"
    exit 1
fi

# Install
info "Installing to $INSTALL_DIR/$BINARY_NAME..."

# Check if binary already exists
if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    warn "Existing installation found at $INSTALL_DIR/$BINARY_NAME"
    info "Updating..."
fi

# Try without sudo first (in case user has write permissions)
if cp "$TMP_DIR/sysmon" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
    : # Success without sudo
elif sudo cp "$TMP_DIR/sysmon" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
    : # Success with sudo
else
    error "Failed to install to $INSTALL_DIR/$BINARY_NAME"
    error "Try running: sudo mkdir -p $INSTALL_DIR && sudo chmod 755 $INSTALL_DIR"
    exit 1
fi

chmod +x "$INSTALL_DIR/$BINARY_NAME"

# Verify installation
if command -v sysmon >/dev/null 2>&1; then
    INSTALLED_VERSION=$(sysmon --version 2>/dev/null || echo "unknown")
    success "Installed successfully! ($INSTALLED_VERSION)"
else
    warn "sysmon is installed but not in your PATH"
    warn "Add $INSTALL_DIR to your PATH or run: export PATH=\"$INSTALL_DIR:\$PATH\""
fi

# Success message
cat << EOF

${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${GREEN}  ✓ sysmon installed successfully!      ${NC}
${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

${CYAN}Quick Start:${NC}
  ${YELLOW}sysmon${NC}              Start with default settings
  ${YELLOW}sysmon --port 8080${NC}  Start on custom port
  ${YELLOW}sysmon --no-browser${NC} Start without opening browser

${CYAN}Configuration:${NC}
  Config file: ~/.config/sysmon/config.toml
  Web UI: http://localhost:3030

${CYAN}Documentation:${NC}
  https://github.com/$REPO

${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}
${BLUE}  Happy monitoring! 🚀                   ${NC}
${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}

EOF

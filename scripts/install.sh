#!/bin/bash
# Sruja DSL Installation Script
# Downloads and installs the latest Sruja DSL binary

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux) OS="Linux" ;;
    Darwin) OS="Darwin" ;;
    *) echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

# Version and download URL
VERSION="${SRUJA_VERSION:-latest}"
INSTALL_DIR="${SRUJA_INSTALL_DIR:-/usr/local/bin}"
BINARY_PATH="${INSTALL_DIR}/sruja"

# GitHub releases URL
REPO="sruja-ai/sruja"
API_URL="https://api.github.com/repos/${REPO}/releases"

echo -e "${GREEN}Installing Sruja DSL...${NC}"
echo -e "OS: ${OS}"
echo -e "Architecture: ${ARCH}"

# Construct archive name based on GoReleaser template: sruja_Linux_x86_64.tar.gz
ARCHIVE_NAME="sruja_${OS}_${ARCH}.tar.gz"

# Get latest release
if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL=$(curl -s "${API_URL}/latest" | grep "browser_download_url.*${ARCHIVE_NAME}" | cut -d '"' -f 4)
else
    DOWNLOAD_URL=$(curl -s "${API_URL}/tags/${VERSION}" | grep "browser_download_url.*${ARCHIVE_NAME}" | cut -d '"' -f 4)
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Error: Could not find download URL for ${ARCHIVE_NAME}${NC}"
    exit 1
fi

echo -e "${YELLOW}Downloading from: ${DOWNLOAD_URL}${NC}"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download archive
curl -L -o "${TMP_DIR}/${ARCHIVE_NAME}" "$DOWNLOAD_URL"

# Extract archive
tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"

if [ ! -f "${TMP_DIR}/sruja" ]; then
    echo -e "${RED}Error: Extraction failed or binary not found in archive${NC}"
    exit 1
fi

# Make executable
chmod +x "${TMP_DIR}/sruja"

# Create install directory if it doesn't exist
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Creating directory: ${INSTALL_DIR}${NC}"
    sudo mkdir -p "$INSTALL_DIR"
fi

# Install binary
echo -e "${YELLOW}Installing to: ${BINARY_PATH}${NC}"
sudo mv "${TMP_DIR}/sruja" "$BINARY_PATH"

# Verify installation
if command -v sruja >/dev/null 2>&1; then
    INSTALLED_VERSION=$(sruja --version 2>&1 || echo "unknown")
    echo -e "${GREEN}✓ Sruja DSL installed successfully!${NC}"
    echo -e "${GREEN}Version: ${INSTALLED_VERSION}${NC}"
    echo ""
    echo -e "Next steps:"
    echo -e "  1. Run: ${YELLOW}sruja --help${NC} to see all commands"
    echo -e "  2. Read: https://sruja-ai.github.io/sruja/docs/getting-started/"
else
    echo -e "${YELLOW}Warning: Binary installed but not found in PATH${NC}"
    echo -e "Make sure ${INSTALL_DIR} is in your PATH"
fi

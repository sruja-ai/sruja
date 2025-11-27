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
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *) echo -e "${RED}Unsupported architecture: $ARCH${NC}"; exit 1 ;;
esac

case "$OS" in
    linux) ;;
    darwin) OS="darwin" ;;
    *) echo -e "${RED}Unsupported OS: $OS${NC}"; exit 1 ;;
esac

# Version and download URL
VERSION="${SRUJA_VERSION:-latest}"
BINARY_NAME="sruja-${OS}-${ARCH}"
INSTALL_DIR="${SRUJA_INSTALL_DIR:-/usr/local/bin}"
BINARY_PATH="${INSTALL_DIR}/sruja"

# GitHub releases URL
REPO="sruja-ai/sruja"
API_URL="https://api.github.com/repos/${REPO}/releases"

echo -e "${GREEN}Installing Sruja DSL...${NC}"
echo -e "OS: ${OS}"
echo -e "Architecture: ${ARCH}"
echo -e "Version: ${VERSION}"

# Get latest release
if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL=$(curl -s "${API_URL}/latest" | grep "browser_download_url.*${BINARY_NAME}" | cut -d '"' -f 4)
else
    DOWNLOAD_URL=$(curl -s "${API_URL}/tags/${VERSION}" | grep "browser_download_url.*${BINARY_NAME}" | cut -d '"' -f 4)
fi

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Error: Could not find download URL for ${BINARY_NAME}${NC}"
    exit 1
fi

echo -e "${YELLOW}Downloading from: ${DOWNLOAD_URL}${NC}"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

# Download binary
curl -L -o "${TMP_DIR}/sruja" "$DOWNLOAD_URL"

if [ ! -f "${TMP_DIR}/sruja" ]; then
    echo -e "${RED}Error: Download failed${NC}"
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
    echo -e "  1. Run: ${YELLOW}sruja init${NC} to create your first project"
    echo -e "  2. Run: ${YELLOW}sruja --help${NC} to see all commands"
    echo -e "  3. Read: https://sruja.dev/docs/quickstart"
else
    echo -e "${YELLOW}Warning: Binary installed but not found in PATH${NC}"
    echo -e "Make sure ${INSTALL_DIR} is in your PATH"
fi


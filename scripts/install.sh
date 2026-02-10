#!/usr/bin/env bash
# Install the Sruja CLI from GitHub Releases.
# Usage: curl -fsSL https://sruja.ai/install.sh | bash
#        curl -fsSL https://sruja.ai/install.sh | bash -s -- sruja-v0.7.7   # or v0.6.1, or omit for latest
set -e

REPO="${SRUJA_REPO:-sruja-ai/sruja}"
VERSION="${1:-latest}"
INSTALL_DIR="${SRUJA_INSTALL_DIR:-$HOME/.local/bin}"

# Resolve latest to the current release tag
if [ "$VERSION" = "latest" ]; then
  if command -v jq &>/dev/null; then
    VERSION=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | jq -r .tag_name)
  else
    VERSION=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
  fi
fi

# Version for asset filename (e.g. sruja-v0.7.6 or v0.6.1 -> 0.7.6 / 0.6.1)
VER="${VERSION#sruja-}"
VER="${VER#v}"

# Detect OS and arch
OS=""
ARCH=""
case "$(uname -s)" in
  Linux)   OS=linux ;;
  Darwin)  OS=darwin ;;
  MINGW*|MSYS*|CYGWIN*) OS=windows ;;
  *)       echo "Unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=aarch64 ;;
  *)       echo "Unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

# Windows: only x86_64 is built
if [ "$OS" = "windows" ]; then
  ARCH=x86_64
fi

if [ "$OS" = "windows" ]; then
  ASSET="sruja-${VER}-${OS}-${ARCH}.zip"
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
  echo "Downloading $URL"
  TMP="$(mktemp -d)"
  trap "rm -rf '$TMP'" EXIT
  curl -fsSL "$URL" -o "$TMP/sruja.zip"
  unzip -o "$TMP/sruja.zip" -d "$TMP"
  mkdir -p "$INSTALL_DIR"
  mv "$TMP/sruja.exe" "$INSTALL_DIR/sruja.exe"
  echo "Installed to $INSTALL_DIR/sruja.exe"
  echo "Ensure $INSTALL_DIR is on your PATH."
  exit 0
fi

ASSET="sruja-${VER}-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
echo "Downloading $URL"
TMP="$(mktemp -d)"
trap "rm -rf '$TMP'" EXIT
curl -fsSL "$URL" -o "$TMP/sruja.tar.gz"
tar xzf "$TMP/sruja.tar.gz" -C "$TMP"
mkdir -p "$INSTALL_DIR"
mv "$TMP/sruja" "$INSTALL_DIR/sruja"
chmod +x "$INSTALL_DIR/sruja"
echo "Installed to $INSTALL_DIR/sruja"
echo "Ensure $INSTALL_DIR is on your PATH (e.g. export PATH=\"$INSTALL_DIR:\$PATH\")."

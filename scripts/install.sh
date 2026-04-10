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

if [ "$OS" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then
  echo "No prebuilt macOS x86_64 (Intel) binaries are published for Sruja." >&2
  echo "Install from source (requires Rust):" >&2
  echo "  cargo install --git https://github.com/${REPO}.git --tag ${VERSION} sruja-cli --locked" >&2
  exit 1
fi

find_release_asset_url() {
  local repo="$1"
  local tag="$2"
  local asset="$3"

  local json
  if ! json="$(curl -fsSL "https://api.github.com/repos/${repo}/releases/tags/${tag}")"; then
    return 1
  fi

  if command -v jq &>/dev/null; then
    echo "$json" | jq -r --arg asset "$asset" '.assets[] | select(.name == $asset) | .browser_download_url' | head -n 1
    return 0
  fi

  local compact
  compact="$(echo "$json" | tr -d '\n' | tr -d ' ')"
  echo "$compact" | sed -n "s/.*\"name\":\"${asset}\"[^}]*\"browser_download_url\":\"\\([^\"]*\\)\".*/\\1/p"
}

print_release_assets() {
  local repo="$1"
  local tag="$2"

  local json
  if ! json="$(curl -fsSL "https://api.github.com/repos/${repo}/releases/tags/${tag}")"; then
    return 0
  fi

  if command -v jq &>/dev/null; then
    echo "$json" | jq -r '.assets[].name' 2>/dev/null || true
    return 0
  fi

  echo "$json" | grep -Eo '"name":[[:space:]]*"[^"]+"' | sed -E 's/.*"name":[[:space:]]*"([^"]+)".*/\1/' | sort -u
}

if [ "$OS" = "windows" ]; then
  ASSET="sruja-${VER}-${OS}-${ARCH}.zip"
  URL="$(find_release_asset_url "$REPO" "$VERSION" "$ASSET" || true)"
  if [ -z "$URL" ] || [ "$URL" = "null" ]; then
    echo "No prebuilt asset '${ASSET}' found on release ${VERSION}." >&2
    echo "Release page: https://github.com/${REPO}/releases/tag/${VERSION}" >&2
    echo "Available assets:" >&2
    print_release_assets "$REPO" "$VERSION" | sed 's/^/  - /' >&2
    echo "If this release was just published, binaries may still be uploading. Try again in a few minutes." >&2
    echo "Fallback (requires Rust):" >&2
    echo "  cargo install --git https://github.com/${REPO}.git --tag ${VERSION} sruja-cli --locked" >&2
    exit 1
  fi
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
URL="$(find_release_asset_url "$REPO" "$VERSION" "$ASSET" || true)"
if [ -z "$URL" ] || [ "$URL" = "null" ]; then
  echo "No prebuilt asset '${ASSET}' found on release ${VERSION}." >&2
  echo "Release page: https://github.com/${REPO}/releases/tag/${VERSION}" >&2
  echo "Available assets:" >&2
  print_release_assets "$REPO" "$VERSION" | sed 's/^/  - /' >&2
  echo "If this release was just published, binaries may still be uploading. Try again in a few minutes." >&2
  echo "Fallback (requires Rust):" >&2
  echo "  cargo install --git https://github.com/${REPO}.git --tag ${VERSION} sruja-cli --locked" >&2
  exit 1
fi
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

#!/bin/bash
# Script to get the latest commit SHA for GitHub Actions
# Usage: ./scripts/get-action-shas.sh actions/setup-node v4.1.0

ACTION_REPO=$1
VERSION=$2

if [ -z "$ACTION_REPO" ] || [ -z "$VERSION" ]; then
  echo "Usage: $0 <action-repo> <version>"
  echo "Example: $0 actions/setup-node v4.1.0"
  exit 1
fi

# Get the SHA for the tag
SHA=$(curl -sL "https://api.github.com/repos/${ACTION_REPO}/git/ref/tags/${VERSION}" | \
  grep -o '"sha":"[^"]*"' | head -1 | sed 's/"sha":"\([^"]*\)"/\1/')

if [ -n "$SHA" ]; then
  echo "SHA for ${ACTION_REPO}@${VERSION}: ${SHA}"
else
  echo "Could not find SHA for ${ACTION_REPO}@${VERSION}"
  echo "Try: curl -sL https://api.github.com/repos/${ACTION_REPO}/releases/latest | grep tag_name"
  exit 1
fi


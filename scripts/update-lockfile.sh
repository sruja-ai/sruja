#!/bin/bash
# scripts/update-lockfile.sh
# Updates package-lock.json to sync with package.json changes
# Run this after making changes to package.json files in the monorepo

set -e

echo "🔄 Updating package-lock.json..."

# Install dependencies to update lock file
npm install

echo "✅ package-lock.json updated"
echo ""
echo "⚠️  Please review the changes and commit:"
echo "   git add package-lock.json"
echo "   git commit -m 'chore: update package-lock.json'"


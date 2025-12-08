#!/usr/bin/env bash
# Serve the Sruja Viewer locally for development using Vite

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VIEWER_DIR="$PROJECT_ROOT/packages/viewer"
PORT="${PORT:-3001}"

echo "Starting Vite preview server for Sruja Viewer..."
echo ""
echo "Viewer will be available at: http://localhost:$PORT/sruja-viewer.umd.cjs"
echo ""
echo "Set in .env.local:"
echo "  SRUJA_VIEWER_URL=http://localhost:$PORT/sruja-viewer.umd.cjs"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

cd "$VIEWER_DIR"
npm run preview:browser


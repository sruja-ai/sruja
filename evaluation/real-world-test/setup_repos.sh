#!/usr/bin/env bash
# Setup script for real-world Sruja testing
# Clones repositories to test Sruja's architecture generation capabilities.
#
# Usage:
#   ./setup_repos.sh           # Quick set: frameworks/libraries (fast, for demos)
#   ./setup_repos.sh --complex # Complex systems: distributed systems, servers, storage
#   ./setup_repos.sh --all     # Both sets
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPOS_DIR="${SCRIPT_DIR}/test-repos"

# Quick set: frameworks and libraries. Small clones, good for demos and CI.
# Format: name|url|description
REPOS_QUICK=(
  "express|https://github.com/expressjs/express.git|Fast, unopinionated, minimalist web framework for Node.js"
  "fastapi|https://github.com/tiangolo/fastapi.git|Modern, fast web framework for building APIs with Python"
  "next.js|https://github.com/vercel/next.js.git|The React Framework for Production"
  "prometheus|https://github.com/prometheus/prometheus.git|The Prometheus monitoring system and time series database"
  "django|https://github.com/django/django.git|The Web framework for perfectionists with deadlines"
)

# Complex systems: customer-facing, admin, ecommerce, and multi-component systems in
# supported languages (Go, JS, Python, TS). Use these to validate Sruja on product-like systems.
REPOS_COMPLEX=(
  "gitea|https://github.com/go-gitea/gitea.git|Self-hosted Git service (Go): web UI, API, Git SSH/HTTP"
  "etcd|https://github.com/etcd-io/etcd.git|Distributed key-value store, Raft consensus, gRPC/HTTP API"
  "caddy|https://github.com/caddyserver/caddy.git|Pluggable web server and reverse proxy with admin API"
  "temporal|https://github.com/temporalio/temporal.git|Workflow orchestration: frontend, history, matching, worker services"
  "minio|https://github.com/minio/minio.git|S3-compatible object storage, erasure coding, gateway mode"
  "react-admin|https://github.com/marmelab/react-admin.git|Admin and dashboard framework (TypeScript/React): CRUD, auth, data providers"
  "saleor|https://github.com/saleor/saleor.git|Headless ecommerce platform (Python/Django): GraphQL API, dashboard, checkout"
)

# Parse flags
MODE="quick"
for arg in "$@"; do
  case "$arg" in
    -h|--help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "  (default)   Quick set: express, fastapi, next.js, prometheus, django (frameworks; fast clone)"
      echo "  --complex   Complex systems: gitea, etcd, caddy, temporal, minio, react-admin, saleor (admin, ecommerce, multi-component)"
      echo "  --all       Both quick and complex"
      echo "  -h, --help  Show this help"
      exit 0
      ;;
    --complex) MODE="complex" ;;
    --all)     MODE="all" ;;
  esac
done

case "$MODE" in
  quick)   REPOS=("${REPOS_QUICK[@]}")   ;;
  complex) REPOS=("${REPOS_COMPLEX[@]}") ;;
  all)     REPOS=("${REPOS_QUICK[@]}" "${REPOS_COMPLEX[@]}") ;;
esac

echo "🚀 Sruja Real-World Test Setup"
echo "=================================================="
echo ""
echo "Mode: $MODE ($(echo "${REPOS[@]}" | wc -w) repositories)"
echo ""

mkdir -p "$REPOS_DIR"

for entry in "${REPOS[@]}"; do
  IFS='|' read -r name url desc <<< "$entry"
  repo_path="${REPOS_DIR}/${name}"

  if [ -d "$repo_path" ]; then
    echo "✓ $name already exists, skipping..."
    continue
  fi

  echo "⬇️  Cloning $name..."
  echo "   $desc"
  if git clone --depth 1 "$url" "$repo_path" 2>/dev/null; then
    echo "✅ Successfully cloned $name"
  else
    echo "❌ Failed to clone $name (check network and URL)"
    exit 1
  fi
  echo ""
done

# Manifest metadata (name -> language|complexity|arch-type); kept in sync with both lists
declare -A REPO_META=(
  [express]="JavaScript|medium|backend-framework"
  [fastapi]="Python|medium|backend-framework"
  [next.js]="TypeScript|high|fullstack-framework"
  [prometheus]="Go|high|distributed-system"
  [django]="Python|high|fullstack-framework"
  [gitea]="Go|high|customer-facing-app"
  [etcd]="Go|high|distributed-kv"
  [caddy]="Go|high|web-server"
  [temporal]="Go|high|workflow-orchestration"
  [minio]="Go|high|object-storage"
  [react-admin]="TypeScript|high|admin-dashboard"
  [saleor]="Python|high|ecommerce"
)

# Build manifest from all repos that exist under REPOS_DIR
MANIFEST="${REPOS_DIR}/MANIFEST.md"
{
  echo "# Test Repositories for Sruja Architecture Generation"
  echo ""
  echo "This directory contains open-source projects for testing Sruja's"
  echo "ability to generate useful architecture documentation."
  echo ""
  echo "## Quick set (frameworks / libraries)"
  echo ""
  n=1
  for entry in "${REPOS_QUICK[@]}"; do
    IFS='|' read -r name url desc <<< "$entry"
    IFS='|' read -r lang complexity arch_type <<< "${REPO_META[$name]:-unknown|medium|backend-framework}"
    echo "### $n. $name"
    echo "- **Description**: $desc"
    echo "- **Language**: $lang"
    echo "- **Complexity**: $complexity"
    echo "- **Architecture Type**: $arch_type"
    echo "- **URL**: $url"
    echo ""
    ((n++)) || true
  done
  echo "## Complex systems (customer-facing / multi-component)"
  echo ""
  for entry in "${REPOS_COMPLEX[@]}"; do
    IFS='|' read -r name url desc <<< "$entry"
    IFS='|' read -r lang complexity arch_type <<< "${REPO_META[$name]:-unknown|high|distributed-system}"
    echo "### $n. $name"
    echo "- **Description**: $desc"
    echo "- **Language**: $lang"
    echo "- **Complexity**: $complexity"
    echo "- **Architecture Type**: $arch_type"
    echo "- **URL**: $url"
    echo ""
    ((n++)) || true
  done
} > "$MANIFEST"

echo "📝 Created manifest at $MANIFEST"
echo ""
echo "=================================================="
echo "✅ Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. cd test-repos/<repo-name>"
echo "2. Use Sruja AI skills to generate architecture"
echo "   Example: Ask your AI assistant to 'Analyze this codebase and create a Sruja architecture DSL'"
echo "3. Review the generated .sruja file"
echo "4. Run evaluation: ./evaluate_architecture.sh <repo-name>"
echo ""
if [ "$MODE" = "quick" ]; then
  echo "Tip: Use ./setup_repos.sh --complex to add complex systems (gitea, etcd, caddy, temporal, minio, react-admin, saleor)"
  echo ""
fi
echo "=================================================="

#!/usr/bin/env bash
# Copy the Sruja architecture skill into each real test repo so that opening
# that repo in Cursor or VS Code shows the skill (no global install required).
#
# Usage: ./prepare_skill_in_real_projects.sh
# Run from evaluation/real-world-test. Operates on all repos in test-repos/.
# Prerequisite: ./setup_repos.sh (or --complex) so test-repos/ is populated.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRUJA_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SKILL_SRC="${SRUJA_ROOT}/skills/sruja-architecture"
REPOS_DIR="${SCRIPT_DIR}/test-repos"

if [ ! -d "$SKILL_SRC" ] || [ ! -f "${SKILL_SRC}/SKILL.md" ]; then
  echo "❌ Sruja skill not found at ${SKILL_SRC}"
  exit 1
fi

if [ ! -d "$REPOS_DIR" ]; then
  echo "❌ test-repos/ not found. Run ./setup_repos.sh first."
  exit 1
fi

count=0
for repo_path in "${REPOS_DIR}"/*/; do
  [ -d "$repo_path" ] || continue
  count=$((count + 1))
done
if [ "$count" -eq 0 ]; then
  echo "❌ test-repos/ is empty. Run ./setup_repos.sh (or --complex) first."
  exit 1
fi

echo "Copying Sruja architecture skill into ${count} repo(s) under test-repos/"
echo ""

for repo_path in "${REPOS_DIR}"/*/; do
  [ -d "$repo_path" ] || continue
  name=$(basename "$repo_path")
  dest_dir="${repo_path}.agents/skills/sruja-architecture"
  mkdir -p "$(dirname "$dest_dir")"
  rm -rf "$dest_dir"
  cp -r "$SKILL_SRC" "$dest_dir"
  echo "  ✓ $name"
done

echo ""
echo "Done. Open any test-repos/<name> in Cursor or VS Code and use /sruja-architecture in chat."

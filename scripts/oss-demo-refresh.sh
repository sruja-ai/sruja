#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRUJA="${SRUJA:-$ROOT/target/release/sruja}"
if [[ ! -x "$SRUJA" ]]; then
  echo "Build sruja first: cargo build -p sruja-cli --release" >&2
  exit 1
fi
"$SRUJA" drift -r "$ROOT/examples/oss-demo/minimal" -f json --structural-only --advisory \
  > "$ROOT/examples/oss-demo/minimal-structural-drift.json"
"$SRUJA" drift -r "$ROOT" -f json --structural-only --advisory \
  | jq '{
    artifact_kind,
    metric_description,
    clean_scan,
    health_score,
    scan_scope: {total_files: .scan_scope.total_files},
    could_not_infer,
    violation_count: (.violations | length),
    sample_violations: (.violations[:3])
  }' > "$ROOT/examples/oss-demo/sruja-repo-structural-excerpt.json"
echo "Updated examples/oss-demo/*.json"

#!/usr/bin/env bash
# Compare two .sruja files (e.g. golden vs generated) — structural stats and deltas.
#
# Usage:
#   ./compare_architecture.sh <golden.sruja> <generated.sruja>
#   ./compare_architecture.sh test-repos/express/architecture.sruja run_results/generated_express.sruja
#
# Output: side-by-side counts, deltas, lint status for both.
# See EVALUATION_METHODOLOGY.md for metrics and how to use.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "${SCRIPT_DIR}/lib.sh"

usage() {
  echo "Usage: $0 <golden.sruja> <generated.sruja>"
  echo ""
  echo "  golden.sruja    Reference (hand-authored) architecture file"
  echo "  generated.sruja Agent-generated or candidate file to compare"
  echo ""
  echo "Example:"
  echo "  $0 test-repos/express/architecture.sruja run_results/generated_express.sruja"
  exit 1
}

[ $# -ge 2 ] || usage
GOLDEN="$1"
GENERATED="$2"

[ -f "$GOLDEN" ]    || { echo "❌ Not found: $GOLDEN"; exit 1; }
[ -f "$GENERATED" ] || { echo "❌ Not found: $GENERATED"; exit 1; }

# Count helpers (normalize to digits)
norm() { echo "$1" | tr -d ' \n'; }
count() {
  local file="$1"
  local pattern="$2"
  grep -c -e "$pattern" "$file" 2>/dev/null || echo "0"
}

# Lint one file; echo "pass", "fail", or "skip"
lint_status() {
  local f="$1"
  local sruja
  sruja=$(find_sruja)
  if [ -z "$sruja" ]; then
    echo "skip"
    return
  fi
  if $sruja lint "$f" >/dev/null 2>&1; then
    echo "pass"
  else
    echo "fail"
  fi
}

echo ""
echo "============================================================"
echo "Compare: golden vs generated"
echo "============================================================"
echo "  Golden:    $GOLDEN"
echo "  Generated: $GENERATED"
echo ""

# Gather stats (portable: no mapfile)
G_SYS=$(norm "$(count "$GOLDEN" '= system')")
G_CON=$(norm "$(count "$GOLDEN" '= container')")
G_COMP=$(norm "$(count "$GOLDEN" '= component')")
G_DB=$(($(norm "$(count "$GOLDEN" '= database')") + $(norm "$(count "$GOLDEN" '= datastore')")))
G_PERS=$(norm "$(count "$GOLDEN" '= person')")
G_REL=$(norm "$(count "$GOLDEN" '->')")

N_SYS=$(norm "$(count "$GENERATED" '= system')")
N_CON=$(norm "$(count "$GENERATED" '= container')")
N_COMP=$(norm "$(count "$GENERATED" '= component')")
N_DB=$(($(norm "$(count "$GENERATED" '= database')") + $(norm "$(count "$GENERATED" '= datastore')")))
N_PERS=$(norm "$(count "$GENERATED" '= person')")
N_REL=$(norm "$(count "$GENERATED" '->')")

echo "  Metric         Golden    Generated   Delta"
echo "  -------------- --------- ----------- ------"
printf "  %-14s %9s %11s %+6d\n" "Systems"       "$G_SYS" "$N_SYS" "$((N_SYS - G_SYS))"
printf "  %-14s %9s %11s %+6d\n" "Containers"    "$G_CON" "$N_CON" "$((N_CON - G_CON))"
printf "  %-14s %9s %11s %+6d\n" "Components"    "$G_COMP" "$N_COMP" "$((N_COMP - G_COMP))"
printf "  %-14s %9s %11s %+6d\n" "Datastores"    "$G_DB" "$N_DB" "$((N_DB - G_DB))"
printf "  %-14s %9s %11s %+6d\n" "Persons"       "$G_PERS" "$N_PERS" "$((N_PERS - G_PERS))"
printf "  %-14s %9s %11s %+6d\n" "Relationships" "$G_REL" "$N_REL" "$((N_REL - G_REL))"
echo ""

LINT_G=$(lint_status "$GOLDEN")
LINT_N=$(lint_status "$GENERATED")
echo "  Lint (golden):    $LINT_G"
echo "  Lint (generated): $LINT_N"
echo ""

# Short summary
DELTA_C=$((N_CON - G_CON))
DELTA_R=$((N_REL - G_REL))
echo "  Summary:"
if [ "$DELTA_C" -gt 0 ]; then
  echo "    Generated has +$DELTA_C containers vs golden"
elif [ "$DELTA_C" -lt 0 ]; then
  echo "    Generated has $DELTA_C containers vs golden"
fi
if [ "$DELTA_R" -gt 0 ]; then
  echo "    Generated has +$DELTA_R relationships vs golden"
elif [ "$DELTA_R" -lt 0 ]; then
  echo "    Generated has $DELTA_R relationships vs golden"
fi
[ "$LINT_N" = "fail" ] && echo "    ⚠ Generated file fails sruja lint — fix before using as baseline."
echo ""
echo "============================================================"
echo ""

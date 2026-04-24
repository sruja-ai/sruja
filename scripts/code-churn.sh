#!/usr/bin/env bash
# Code Churn Tracker - Measures lines rewritten/deleted within 2 weeks
# Based on talks: "Speed Without Comprehension Is a Delayed Disaster"
# Run: ./scripts/code-churn.sh [days]

set -e

DAYS=${1:-14}
REPO_ROOT="${REPO_ROOT:-.}"

echo "📊 Code Churn Report (last ${DAYS} days)"
echo "========================================"

# Get commits in date range
# macOS/BSD date compatibility
if date -v-14d +%Y-%m-%d 2>/dev/null >/dev/null; then
    SINCE_DATE=$(date -v-${DAYS}d +%Y-%m-%d)
else
    SINCE_DATE=$(date -d "${DAYS} days ago" +%Y-%m-%d)
fi

echo "📅 Date range: ${SINCE_DATE} to $(date +%Y-%m-%d)"
echo ""

# Lines added (insertions)
LINES_ADDED=$(git log --since="${SINCE_DATE}" --oneline --shortstat -- . | awk '{add += $4} END {print add + 0}')

# Lines deleted
LINES_DELETED=$(git log --since="${SINCE_DATE}" --oneline --shortstat -- . | awk '{del += $6} END {print del + 0}')

# Total lines changed
LINES_TOTAL=$((LINES_ADDED + LINES_DELETED))

echo "📈 Statistics:"
echo "  Lines added:     ${LINES_ADDED}"
echo "  Lines deleted:  ${LINES_DELETED}"
echo "  Total changes: ${LINES_TOTAL}"
echo ""

# Calculate churn rate (changes / total codebase lines)
TOTAL_LINES=$(find "${REPO_ROOT}"/crates -name '*.rs' -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
if [ "$TOTAL_LINES" -gt 0 ]; then
    CHURN_RATE=$(awk "BEGIN {printf \"%.1f\", (${LINES_TOTAL} / ${TOTAL_LINES}) * 100}")
    echo "📊 Churn Rate: ${CHURN_RATE}% (${LINES_TOTAL} / ${TOTAL_LINES} total lines)"
else
    echo "📊 Churn Rate: N/A (could not calculate total lines)"
fi

echo ""

# Churn by author (top 5)
echo "👤 Top Contributors by Churn:"
git log --since="${SINCE_DATE}" --format='%aN' -- . | sort | uniq -c | sort -rn | head -5 | while read count author; do
    echo "  ${author}: ${count} commits"
done

echo ""

# Flag high churn (threshold: 7.9% from talks)
if [ "$CHURN_RATE" != "N/A" ] && [ "${CHURN_RATE}" != "0.0" ]; then
    if (( $(echo "$CHURN_RATE > 5.0" | bc -l 2>/dev/null || echo "0") )); then
        echo "⚠️  WARNING: High code churn detected (${CHURN_RATE}%)"
        echo "   This may indicate comprehension debt or rapid AI-generated code growth."
    else
        echo "✅ Code churn is within acceptable range (< 5%)"
    fi
fi

echo ""
echo "Run './scripts/code-churn.sh 7' for weekly report"
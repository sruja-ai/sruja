#!/bin/bash
# Script to iteratively improve diagram quality based on e2e test scores
# 
# Usage:
#   ./scripts/improve-diagram-quality.sh [iterations]
# 
# This script:
# 1. Runs quality e2e tests
# 2. Analyzes quality scores
# 3. Suggests improvements (or can be extended to auto-apply)
# 4. Tracks progress over iterations

set -e

ITERATIONS=${1:-3}
RESULTS_DIR="tests/results/quality-iterative"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "🚀 Starting iterative diagram quality improvement"
echo "   Iterations: $ITERATIONS"
echo "   Results dir: $RESULTS_DIR"
echo ""

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"

# Function to run quality tests
run_quality_tests() {
    echo "📊 Running quality tests (iteration $1/$ITERATIONS)..."
    npm run test:e2e -- tests/diagram-quality-iterative.spec.ts 2>&1 | tee "$RESULTS_DIR/run-$1.log"
}

# Function to extract latest scores from history
get_latest_scores() {
    local file="$1"
    if [ -f "$file" ]; then
        node -e "
            const fs = require('fs');
            const data = JSON.parse(fs.readFileSync('$file', 'utf-8'));
            const latest = data.runs[data.runs.length - 1];
            if (latest) {
                console.log(JSON.stringify({
                    score: latest.metrics.score,
                    crossings: latest.metrics.edgeCrossings,
                    overlaps: latest.metrics.nodeOverlaps,
                    alignment: latest.metrics.rankAlignment
                }));
            }
        " 2>/dev/null || echo "{}"
    else
        echo "{}"
    fi
}

# Function to compare scores and suggest improvements
analyze_quality() {
    echo ""
    echo "📈 Quality Analysis:"
    echo "==================="
    
    for example_file in ecommerce_platform.sruja project_ecommerce.sruja; do
        history_file="$RESULTS_DIR/${example_file}-history.json"
        
        if [ ! -f "$history_file" ]; then
            echo "⚠️  No history found for $example_file"
            continue
        fi
        
        node -e "
            const fs = require('fs');
            const data = JSON.parse(fs.readFileSync('$history_file', 'utf-8'));
            const runs = data.runs || [];
            const baseline = data.baseline;
            const latest = runs[runs.length - 1];
            
            if (!latest) {
                console.log('⚠️  No runs found for $example_file');
                process.exit(0);
            }
            
            const metrics = latest.metrics;
            console.log('\n📊 $example_file:');
            console.log(\`   Score: \${metrics.score.toFixed(3)} (target: ≥0.85)\`);
            console.log(\`   Edge Crossings: \${metrics.edgeCrossings} (target: ≤5)\`);
            console.log(\`   Node Overlaps: \${metrics.nodeOverlaps} (target: 0)\`);
            console.log(\`   Rank Alignment: \${(metrics.rankAlignment * 100).toFixed(1)}% (target: ≥95%)\`);
            
            if (baseline) {
                const improvement = ((metrics.score - baseline.score) * 100).toFixed(2);
                console.log(\`   vs Baseline: \${improvement > 0 ? '+' : ''}\${improvement}%\`);
            }
            
            // Suggest improvements
            const suggestions = [];
            if (metrics.score < 0.85) {
                suggestions.push('Score below target (0.85)');
            }
            if (metrics.edgeCrossings > 5) {
                suggestions.push(\`Too many edge crossings (\${metrics.edgeCrossings} > 5)\`);
            }
            if (metrics.nodeOverlaps > 0) {
                suggestions.push(\`Node overlaps detected (\${metrics.nodeOverlaps})\`);
            }
            if (metrics.rankAlignment < 0.95) {
                suggestions.push(\`Poor rank alignment (\${(metrics.rankAlignment * 100).toFixed(1)}% < 95%)\`);
            }
            
            if (suggestions.length > 0) {
                console.log('   🔧 Suggestions:');
                suggestions.forEach(s => console.log(\`      - \${s}\`));
            } else {
                console.log('   ✅ All quality targets met!');
            }
        " 2>/dev/null || echo "   ⚠️  Failed to analyze"
    done
    echo ""
}

# Run iterations
for i in $(seq 1 $ITERATIONS); do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Iteration $i/$ITERATIONS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Run tests
    run_quality_tests $i
    
    # Analyze results
    analyze_quality
    
    # If not last iteration, wait a bit (allows for manual code changes if needed)
    if [ $i -lt $ITERATIONS ]; then
        echo "⏳ Waiting 5 seconds before next iteration..."
        echo "   (You can make code changes now if needed)"
        sleep 5
    fi
done

# Generate final summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Final Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

node -e "
    const fs = require('fs');
    const path = require('path');
    const resultsDir = '$RESULTS_DIR';
    
    console.log('\n📊 Quality Improvement Summary:\n');
    
    for (const example of ['ecommerce_platform.sruja', 'project_ecommerce.sruja']) {
        const historyFile = path.join(resultsDir, \`\${example}-history.json\`);
        if (!fs.existsSync(historyFile)) {
            console.log(\`⚠️  \${example}: No data\`);
            continue;
        }
        
        const data = JSON.parse(fs.readFileSync(historyFile, 'utf-8'));
        const runs = data.runs || [];
        const baseline = data.baseline;
        
        if (runs.length === 0) {
            console.log(\`⚠️  \${example}: No runs\`);
            continue;
        }
        
        const first = runs[0].metrics;
        const latest = runs[runs.length - 1].metrics;
        
        console.log(\`📈 \${example}:\`);
        console.log(\`   Runs: \${runs.length}\`);
        console.log(\`   Baseline Score: \${baseline?.score.toFixed(3) || first.score.toFixed(3)}\`);
        console.log(\`   Latest Score: \${latest.score.toFixed(3)}\`);
        
        if (baseline) {
            const totalImprovement = ((latest.score - baseline.score) * 100).toFixed(2);
            console.log(\`   Total Improvement: \${totalImprovement > 0 ? '+' : ''}\${totalImprovement}%\`);
        }
        
        const trend = runs.length > 1 && latest.score > first.score ? '📈 Improving' : '➡️  Stable';
        console.log(\`   Trend: \${trend}\`);
        console.log('');
    }
    
    console.log('✅ Analysis complete! Check $RESULTS_DIR for detailed reports.');
" 2>/dev/null || echo "⚠️  Failed to generate summary"

echo ""
echo "✨ Done! Check $RESULTS_DIR for detailed results."


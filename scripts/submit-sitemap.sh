#!/bin/bash
# submit-sitemap.sh
# Manual script to submit sitemap to search engines
# Can also be run in CI/CD

set -e

SITE_URL="${1:-https://sruja.ai}"
SITEMAP_URL="${SITE_URL}/sitemap.xml"

echo "=== Submitting Sitemap to Search Engines ==="
echo "Sitemap URL: $SITEMAP_URL"
echo ""

# Google
echo "📤 Submitting to Google..."
if curl -s "https://www.google.com/ping?sitemap=${SITEMAP_URL}" > /dev/null; then
  echo "✅ Successfully pinged Google"
else
  echo "❌ Failed to ping Google"
fi

# Bing
echo "📤 Submitting to Bing..."
if curl -s "https://www.bing.com/ping?sitemap=${SITEMAP_URL}" > /dev/null; then
  echo "✅ Successfully pinged Bing"
else
  echo "❌ Failed to ping Bing"
fi

# Yandex
echo "📤 Submitting to Yandex..."
if curl -s "https://webmaster.yandex.com/ping?sitemap=${SITEMAP_URL}" > /dev/null; then
  echo "✅ Successfully pinged Yandex"
else
  echo "❌ Failed to ping Yandex"
fi

echo ""
echo "=== Submission Complete ==="
echo ""
echo "Note: These pings notify search engines about sitemap updates."
echo "For full automation with Search Console API, see docs/SEO_SUBMISSION_AUTOMATION.md"


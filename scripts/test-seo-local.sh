#!/bin/bash
# test-seo-local.sh
# Builds Hugo site and checks for SEO elements in generated HTML

cd "$(dirname "$0")/../learn" || exit 1

echo "=== Building Hugo Site ==="
hugo --minify --quiet || { echo "Build failed!"; exit 1; }

echo ""
echo "=== Checking SEO Elements ==="
echo ""

# Check for meta tags
echo "✓ Meta Tags:"
og_count=$(grep -r "og:title" public/*.html 2>/dev/null | wc -l | tr -d ' ')
twitter_count=$(grep -r "twitter:card" public/*.html 2>/dev/null | wc -l | tr -d ' ')
canonical_count=$(grep -r 'rel="canonical"' public/*.html 2>/dev/null | wc -l | tr -d ' ')
echo "  - Open Graph tags found in: $og_count pages"
echo "  - Twitter Card tags found in: $twitter_count pages"
echo "  - Canonical URLs found in: $canonical_count pages"

# Check for structured data
echo ""
echo "✓ Structured Data (JSON-LD):"
jsonld_count=$(grep -r "application/ld+json" public/*.html 2>/dev/null | wc -l | tr -d ' ')
echo "  - JSON-LD schemas found in: $jsonld_count pages"

# Check sitemap
echo ""
echo "✓ Sitemap:"
if [ -f "public/sitemap.xml" ]; then
  url_count=$(grep -c "<loc>" public/sitemap.xml 2>/dev/null || echo "0")
  echo "  - Sitemap exists with $url_count URLs"
else
  echo "  - ❌ Sitemap not found!"
fi

# Check robots.txt
echo ""
echo "✓ robots.txt:"
if [ -f "public/robots.txt" ]; then
  echo "  - robots.txt exists"
  if grep -q "Sitemap" public/robots.txt; then
    echo "  - Sitemap reference found"
  fi
else
  echo "  - ❌ robots.txt not found!"
fi

# Sample page check
echo ""
echo "=== Sample Page Check (index.html) ==="
if [ -f "public/index.html" ]; then
  echo "Checking: public/index.html"
  if grep -q "og:title" public/index.html; then
    echo "  ✓ Open Graph tags present"
  else
    echo "  ❌ Open Graph tags missing"
  fi
  if grep -q "application/ld+json" public/index.html; then
    echo "  ✓ Structured data present"
  else
    echo "  ❌ Structured data missing"
  fi
fi

echo ""
echo "=== Next Steps ==="
echo "1. Test online: https://search.google.com/test/rich-results"
echo "2. Test social sharing: https://developers.facebook.com/tools/debug/"
echo "3. Submit sitemap: https://search.google.com/search-console"


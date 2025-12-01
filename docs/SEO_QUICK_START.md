# SEO Quick Start Guide

Quick reference for implementing SEO improvements.

## 🚀 Immediate Actions (Do Today)

### 1. Test What We Built

```bash
# Build the site
cd learn
hugo --minify

# Run our test script
../scripts/test-seo-local.sh
```

This checks if SEO tags are generating correctly.

### 2. Test Online (5 minutes)

1. **Google Rich Results Test**:
   - Visit: https://search.google.com/test/rich-results
   - Enter: `https://sruja.ai/` (or your deployed URL)
   - Should see: Organization, SoftwareApplication schemas ✓

2. **Facebook Debugger**:
   - Visit: https://developers.facebook.com/tools/debug/
   - Enter: `https://sruja.ai/`
   - Click "Scrape Again"
   - Should see: Title, description, image preview ✓

### 3. Find Pages Needing Descriptions

```bash
# Run helper script
./scripts/check-missing-descriptions.sh
```

This lists all pages without descriptions. Add descriptions to important pages first.

## 📝 Adding Descriptions (Template)

For any page missing a description, add to frontmatter:

```yaml
---
title: "Page Title"
description: "150-160 character description with key terms. Keep it compelling and informative."
keywords: ["keyword1", "keyword2"]  # Optional but helpful
---
```

**Priority pages**:
- All course lessons
- All tutorials  
- All concept docs
- Course/module index pages

## 🎯 This Week

### Day 1-2: Testing
- [ ] Test SEO tags locally
- [ ] Test online validators
- [ ] Fix any errors found

### Day 3-4: Content
- [ ] Run description checker script
- [ ] Add descriptions to top 20 most important pages
- [ ] Create default OG image (1200x630px)

### Day 5: Search Console
- [ ] Set up Google Search Console
- [ ] Submit sitemap
- [ ] Request indexing for key pages

## 📋 Weekly Checklist

- [ ] Check Google Search Console for errors
- [ ] Review page performance
- [ ] Add descriptions to new content
- [ ] Test new pages with validators

## 🛠️ Useful Commands

```bash
# Build and check
cd learn && hugo --minify && ../scripts/test-seo-local.sh

# Find pages without descriptions
./scripts/check-missing-descriptions.sh

# Check sitemap
grep -c "<loc>" learn/public/sitemap.xml

# Validate HTML locally (if you have htmlproofer)
htmlproofer learn/public --disable-external
```

## 🔗 Important Links

- **Google Search Console**: https://search.google.com/search-console
- **Rich Results Test**: https://search.google.com/test/rich-results
- **Facebook Debugger**: https://developers.facebook.com/tools/debug/
- **Twitter Validator**: https://cards-dev.twitter.com/validator
- **PageSpeed Insights**: https://pagespeed.web.dev/

## 📚 Full Documentation

See `docs/SEO_IMPLEMENTATION_GUIDE.md` for:
- Detailed instructions
- Advanced features (search, breadcrumbs)
- Troubleshooting
- Content strategy

## ✅ What's Already Done

- ✅ Meta tags (Open Graph, Twitter Cards)
- ✅ Structured data (JSON-LD)
- ✅ robots.txt
- ✅ SEO configuration
- ✅ Descriptions on key index pages

## ⚠️ Common Issues

**Tags not showing?**
- Clear cache: `hugo --cleanDestinationDir`
- Rebuild: `hugo --minify`
- Check partial is included

**Structured data errors?**
- Use Google's Rich Results Test
- Check JSON-LD syntax
- Ensure dates are ISO 8601 format

**Sitemap missing?**
- Check `baseURL` in hugo.toml
- Verify Hugo generates sitemap
- Check file permissions


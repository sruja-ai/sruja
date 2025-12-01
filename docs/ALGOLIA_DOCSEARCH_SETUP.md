# Algolia DocSearch Setup Guide

## ✅ Free for Open Source Documentation

**Algolia DocSearch is completely FREE** for open-source documentation sites! This is a special program run by Algolia to help the open-source community.

## 🎯 What is DocSearch?

DocSearch is Algolia's free search service specifically for:
- Open-source projects
- Technical documentation sites
- Developer documentation
- API documentation

**Key Benefits:**
- ✅ 100% free (no credit card required)
- ✅ Fully managed (they crawl and index your site)
- ✅ Automatic updates (re-indexes when your site changes)
- ✅ Great UX (fast, typo-tolerant search)
- ✅ Analytics included

## 📋 Eligibility Requirements

To qualify for free DocSearch, your site must:
1. ✅ Be open-source (your code is publicly available)
2. ✅ Have public documentation
3. ✅ Be a technical documentation site
4. ✅ Not be commercial documentation (paid products/services)

**Sruja qualifies!** ✓
- Open-source project: ✅
- Public docs: ✅
- Technical documentation: ✅

## 🚀 How to Apply

### Step 1: Apply Online
1. Go to: https://docsearch.algolia.com/apply/
2. Fill out the application form:
   - **Your repository**: `https://github.com/sruja-ai/sruja`
   - **Documentation URL**: `https://sruja.ai`
   - **Email**: Your contact email
   - **Site description**: "Sruja architecture-as-code language documentation, courses, and tutorials"
   - **Technology**: Hugo

### Step 2: Wait for Approval
- Usually takes 1-3 business days
- They'll email you with approval or questions
- Most open-source projects get approved quickly

### Step 3: Receive Configuration
Once approved, Algolia will send you:
- **API Key** (search-only, public key)
- **Index Name** (e.g., `sruja-ai`)
- **Config ID** (for crawling)
- **Integration code** (HTML/JS snippet)

### Step 4: Integrate into Hugo

Add to your Hugo site:

**1. Add CSS to head** (`learn/layouts/partials/docs/inject/head.html`):
```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@docsearch/css@3" />
```

**2. Add search UI** (where you want the search bar, e.g., header):
```html
<div id="docsearch"></div>
```

**3. Add JavaScript** (before closing `</body>` or in footer):
```html
<script src="https://cdn.jsdelivr.net/npm/@docsearch/js@3"></script>
<script>
  docsearch({
    appId: 'YOUR_APP_ID',
    apiKey: 'YOUR_SEARCH_API_KEY',
    indexName: 'YOUR_INDEX_NAME',
    container: '#docsearch',
    placeholder: 'Search docs...',
  });
</script>
```

## 🔄 Automatic Updates

The best part: **Algolia automatically crawls and updates your index!**
- They crawl your site regularly (usually daily)
- Updates happen automatically when you publish new content
- No manual re-indexing needed
- No maintenance required

## 🎨 Customization

### Styling
DocSearch uses CSS variables for theming. Add to your CSS:
```css
:root {
  --docsearch-primary-color: #2563eb;
  --docsearch-text-color: #1f2937;
  --docsearch-modal-background: #ffffff;
  --docsearch-searchbox-shadow: inset 0 0 0 2px var(--docsearch-primary-color);
}
```

### Configuration Options
```javascript
docsearch({
  appId: 'YOUR_APP_ID',
  apiKey: 'YOUR_API_KEY',
  indexName: 'YOUR_INDEX_NAME',
  container: '#docsearch',
  
  // Customization options
  placeholder: 'Search documentation...',
  translations: {
    button: {
      buttonText: 'Search',
    },
    modal: {
      searchBox: {
        resetButtonTitle: 'Clear',
        resetButtonAriaLabel: 'Clear',
      },
    },
  },
  
  // Customize what gets indexed
  // Algolia handles this via their config, but you can adjust
});
```

## 📊 Analytics

Algolia provides search analytics:
- Search queries
- Popular searches
- No-results queries (help identify content gaps)
- Access via Algolia dashboard

## 🔗 Alternative: Self-Hosted Search

If you prefer not to use Algolia (or while waiting for approval), consider:

### Option 1: Pagefind (Recommended Alternative)
- ✅ 100% free and open-source
- ✅ Self-hosted (runs locally)
- ✅ Zero dependencies
- ✅ Fast and lightweight
- ⚠️ You manage indexing yourself

See [SEO Implementation Guide](./SEO_IMPLEMENTATION_GUIDE.md) for Pagefind setup.

### Option 2: Lunr.js
- ✅ Client-side search (no server needed)
- ✅ No external dependencies
- ⚠️ Requires generating search index during build
- ⚠️ Larger JavaScript bundle

## 📝 Application Template

When applying, you can use this template:

```
Repository: https://github.com/sruja-ai/sruja
Documentation URL: https://sruja.ai
Email: [your-email]
Description: Sruja is an architecture-as-code language for defining, 
visualizing, and validating software architecture. The documentation 
includes courses on system design, tutorials, API reference, and 
examples. Built with Hugo.
Technology: Hugo
Expected traffic: Low to Medium
```

## ✅ Checklist

- [ ] Apply at https://docsearch.algolia.com/apply/
- [ ] Wait for approval (1-3 days)
- [ ] Receive configuration email
- [ ] Add DocSearch CSS to head
- [ ] Add search UI container
- [ ] Add DocSearch JavaScript
- [ ] Test search functionality
- [ ] Customize styling (optional)
- [ ] Monitor search analytics

## 🆘 Troubleshooting

**Not approved?**
- Make sure your repo is clearly open-source
- Ensure docs are publicly accessible
- Wait a few days and reapply if needed
- Contact them if you have questions

**Search not working?**
- Verify API keys are correct
- Check browser console for errors
- Ensure container ID matches
- Verify index name is correct

**Content not appearing?**
- Wait 24-48 hours for initial crawl
- Check Algolia dashboard for crawl status
- Verify your site is accessible to crawlers
- Contact Algolia support if issues persist

## 🎉 Benefits Summary

| Feature | DocSearch | Pagefind | Lunr.js |
|---------|-----------|----------|---------|
| **Cost** | Free | Free | Free |
| **Setup** | Easy | Medium | Hard |
| **Maintenance** | None | Manual | Manual |
| **Speed** | Fast | Fast | Medium |
| **Analytics** | Yes | No | No |
| **Typo tolerance** | Excellent | Good | Basic |
| **Dependencies** | External CDN | Self-hosted | Client-side |

**Recommendation**: Start with DocSearch (it's free and managed), fall back to Pagefind if needed.

## 📚 Resources

- **DocSearch Application**: https://docsearch.algolia.com/apply/
- **DocSearch Docs**: https://docsearch.algolia.com/docs/
- **Integration Guide**: https://docsearch.algolia.com/docs/DocSearch-v3
- **Example Integration**: https://docsearch.algolia.com/docs/DocSearch-v3#integration


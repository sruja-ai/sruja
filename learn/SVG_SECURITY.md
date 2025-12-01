# SVG Export Security Measures

## Overview

The SVG export feature in the Sruja playground generates interactive SVG files with embedded JavaScript for C4 model navigation, requirements, ADRs, and technology documentation. To ensure security when sharing these SVGs, we implement multiple layers of sanitization.

## Security Concerns

SVG files with embedded JavaScript can pose security risks:

1. **XSS Attacks**: Inline scripts can execute malicious code
2. **Event Handlers**: Attributes like `onclick`, `onload` can trigger code execution
3. **Data Exfiltration**: Scripts can send data to external servers
4. **DOM Manipulation**: Scripts can modify the page structure
5. **External Resources**: Loading external scripts or resources

## Security Measures Implemented

### 1. DOMPurify Sanitization

We use [DOMPurify](https://github.com/cure53/DOMPurify), a battle-tested HTML/SVG sanitizer, with strict configuration:

```typescript
const SVG_CONFIG = {
  FORBID_TAGS: ['script', 'iframe', 'object', 'embed', 'link', 'style'],
  FORBID_ATTR: [
    'onerror', 'onload', 'onclick', 'onmouseover', 'onmouseout', 
    'onmousedown', 'onmouseup', 'onfocus', 'onblur', 'onchange',
    'onsubmit', 'onreset', 'onselect', 'onkeydown', 'onkeypress',
    'onkeyup', 'data-content-id', 'data-level', 'data-filter'
  ],
  ALLOW_DATA_ATTR: false,
  ALLOW_UNKNOWN_PROTOCOLS: false
};
```

### 2. Additional Sanitization Steps

After DOMPurify, we perform additional security checks:

- **Remove all `<script>` tags**: Even if they slip through DOMPurify
- **Remove all event handlers**: Strip any attribute starting with `on`
- **Remove data-* attributes**: Prevent XSS via data attributes
- **Sanitize style attributes**: Prevent CSS injection

### 3. Static SVG Output

The sanitized SVG is **static** - all interactive features (level switching, content loading) are removed. This ensures:

- No JavaScript execution
- No event handlers
- No dynamic content loading
- Safe to share publicly

## Usage in Playground

When users select "Sruja Format" in the playground:

1. The architecture is compiled to interactive SVG (with JavaScript)
2. The SVG is immediately sanitized using `sanitizeSvg()`
3. Only the sanitized, static version is displayed
4. Users can download/share the safe SVG

## Best Practices for Sharing

### ✅ Safe to Share

- SVGs generated through the playground (automatically sanitized)
- SVGs exported via CLI and then sanitized
- Static SVG files without scripts

### ⚠️ Use with Caution

- Interactive SVGs with embedded JavaScript (only for trusted environments)
- SVGs from untrusted sources (always sanitize first)

## Implementation Details

### Sanitization Function

```typescript
export function sanitizeSvg(svg: string): string {
  // 1. DOMPurify sanitization
  let sanitized = DOMPurify.sanitize(trimmed, SVG_CONFIG);
  
  // 2. Additional security: Remove scripts
  const scripts = doc.querySelectorAll('script');
  scripts.forEach(script => script.remove());
  
  // 3. Remove event handlers
  allElements.forEach(el => {
    Array.from(el.attributes).forEach(attr => {
      if (attr.name.startsWith('on')) {
        el.removeAttribute(attr.name);
      }
      if (attr.name.startsWith('data-')) {
        el.removeAttribute(attr.name);
      }
    });
  });
  
  return sanitized;
}
```

### Playground Integration

The playground automatically sanitizes all SVG output:

```typescript
const output = processCompileOutput(result);
if (output) {
  // Output is already sanitized via sanitizeSvg()
  setVisualOutput(output);
}
```

## Testing Security

To verify security measures:

1. **Test with malicious SVG**: Try injecting `<script>alert('XSS')</script>`
2. **Test event handlers**: Try adding `onclick="alert('XSS')"`
3. **Test data attributes**: Try adding `data-content-id="javascript:alert('XSS')"`

All should be removed by the sanitizer.

## Future Enhancements

For interactive features in shared SVGs, consider:

1. **External Event Handlers**: Re-implement interactivity in React/TypeScript
2. **CSP Headers**: Use Content Security Policy to block inline scripts
3. **Sandboxed iframes**: Display interactive SVGs in sandboxed iframes
4. **Server-side Sanitization**: Additional server-side validation

## References

- [DOMPurify Documentation](https://github.com/cure53/DOMPurify)
- [OWASP XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)
- [SVG Security Best Practices](https://www.w3.org/TR/SVG2/security.html)


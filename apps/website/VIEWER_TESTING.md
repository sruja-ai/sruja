# Viewer App Testing Guide

## Quick Start

### 1. Start the Development Server

```bash
cd apps/website
npm install  # If not already done
npm run dev
```

The server will start at `http://localhost:4321` (or the port shown in terminal).

### 2. Access the Viewer

Open your browser and navigate to:
```
http://localhost:4321/viewer
```

## Testing Scenarios

### Test 1: Empty Viewer (Default State)
- **URL**: `http://localhost:4321/viewer`
- **Expected**: Empty editor, no viewer content, no HTML preview
- **Action**: Type DSL code in the editor

### Test 2: Basic DSL Editing
- **URL**: `http://localhost:4321/viewer`
- **Steps**:
  1. Paste this DSL in the editor:
  ```sruja
  architecture "Test System" {
      system App "My App" {
          container Web "Web Server"
          datastore DB "Database"
      }
      person User "User"

      User -> Web "Visits"
      Web -> DB "Reads/Writes"
  }
  ```
  2. Wait for auto-parsing (should happen automatically)
  3. **Verify**:
     - Viewer shows diagram with User, Web, DB nodes
     - HTML preview updates with the architecture
     - No errors in console

### Test 3: URL Parameter with LZ-String Compressed Code

First, compress your DSL code using LZ-String:

```javascript
// In browser console or Node.js
const LZString = require('lz-string'); // or import in browser
const dsl = `architecture "Test System" {
    system App "My App" {
        container Web "Web Server"
        datastore DB "Database"
    }
    person User "User"

    User -> Web "Visits"
    Web -> DB "Reads/Writes"
}`;

const compressed = LZString.compressToBase64(dsl);
console.log(compressed);
// Use this in URL: /viewer?code=<compressed-value>
```

- **URL**: `http://localhost:4321/viewer?code=<compressed-base64>`
- **Expected**: 
  - Code loads automatically in editor
  - Viewer displays diagram
  - HTML preview shows generated HTML

### Test 4: Hash Parameter with Compressed Code

- **URL**: `http://localhost:4321/viewer#code=<compressed-base64>`
- **Expected**: Same as Test 3, but using hash instead of query parameter

### Test 5: JSON Data Parameter

- **URL**: `http://localhost:4321/viewer?data=<url-encoded-json>`
- **Example JSON**:
```json
{
  "architecture": {
    "name": "Test System",
    "systems": [{
      "id": "App",
      "label": "My App",
      "containers": [{
        "id": "Web",
        "label": "Web Server"
      }],
      "datastores": [{
        "id": "DB",
        "label": "Database"
      }]
    }],
    "persons": [{
      "id": "User",
      "label": "User"
    }],
    "relations": [
      {"from": "User", "to": "Web", "label": "Visits"},
      {"from": "Web", "to": "DB", "label": "Reads/Writes"}
    ]
  }
}
```
- **Expected**: JSON loads, converts to DSL (if WASM ready), displays in viewer

### Test 6: URL to Fetch JSON

- **URL**: `http://localhost:4321/viewer?url=<url-to-json-file>`
- **Expected**: Fetches JSON from URL, loads into viewer

### Test 7: View Modes

Test all view mode buttons:
- **Split View**: Editor on left, Viewer on right
- **Editor Only**: Full editor
- **Viewer Only**: Full viewer
- **HTML Preview**: Shows generated HTML in iframe

### Test 8: Live Updates

1. Start with split view
2. Type DSL in editor
3. **Verify**:
   - Viewer updates automatically as you type
   - HTML preview updates automatically
   - No page refresh needed

### Test 9: Error Handling

1. Type invalid DSL:
```sruja
architecture "Test" {
    invalid syntax here
}
```
2. **Verify**:
   - Error message appears in toolbar
   - Viewer doesn't break
   - HTML preview doesn't update

### Test 10: WASM Loading

- **Expected**: 
  - "Loading WASM..." message appears briefly on first load
  - WASM loads successfully
  - No console errors

## Example Test DSL Code

### Simple Web App
```sruja
architecture "Simple Web App" {
    system App "Web Application" {
        container Web "Web Server"
        datastore DB "Database"
    }
    person User "End User"

    User -> Web "Uses"
    Web -> DB "Stores data in"
}
```

### Multi-System Architecture
```sruja
architecture "E-commerce Platform" {
    system Shop "Online Shop" {
        container WebApp "Web Application"
        container API "API Server"
        datastore Catalog "Product Catalog"
        datastore Orders "Order Database"
    }
    
    system Payment "Payment Gateway" {
        container PaymentAPI "Payment API"
    }
    
    person Customer "Customer"
    person Admin "Administrator"

    Customer -> WebApp "Browses"
    WebApp -> API "Calls"
    API -> Catalog "Reads from"
    API -> Orders "Writes to"
    API -> PaymentAPI "Processes payments via"
    Admin -> API "Manages"
}
```

## Browser Console Testing

Open browser DevTools console and test compression:

```javascript
// Test LZ-String compression
const dsl = `architecture "Test" {
    system App "My App" {
        container Web "Web Server"
    }
    person User "User"
    User -> Web "Uses"
}`;

// Compress
const compressed = LZString.compressToBase64(dsl);
console.log('Compressed:', compressed);

// Test URL
const url = `/viewer?code=${encodeURIComponent(compressed)}`;
console.log('Test URL:', url);

// Decompress (verify)
const decompressed = LZString.decompressFromBase64(compressed);
console.log('Decompressed matches:', decompressed === dsl);
```

## Common Issues & Solutions

### Issue: Viewer not loading
- **Check**: WASM files exist at `/wasm/sruja.wasm` and `/wasm/wasm_exec.js`
- **Solution**: Ensure WASM files are in `apps/website/public/wasm/`

### Issue: Code parameter not working
- **Check**: Code is LZ-String compressed base64, not plain text
- **Solution**: Use `LZString.compressToBase64()` before encoding in URL

### Issue: HTML preview blank
- **Check**: Console for errors
- **Solution**: Ensure CDN URLs are accessible or use local bundles

### Issue: Viewer not updating
- **Check**: WASM is loaded (`wasmApiRef.current` exists)
- **Solution**: Wait for WASM to load before typing

## Automated Testing

To create a test script:

```bash
# Test viewer loads
curl http://localhost:4321/viewer | grep -q "Interactive Viewer" && echo "✓ Viewer page loads"

# Test with code parameter (replace with actual compressed code)
curl "http://localhost:4321/viewer?code=..." | grep -q "architecture" && echo "✓ Code parameter works"
```

## Performance Testing

1. **Large DSL**: Test with 100+ nodes
2. **Rapid typing**: Type quickly and verify updates don't lag
3. **Multiple switches**: Switch between view modes rapidly
4. **Memory**: Check for memory leaks after extended use

## Checklist

- [ ] Viewer page loads without errors
- [ ] Empty state displays correctly
- [ ] Editor accepts DSL input
- [ ] Viewer updates on DSL change
- [ ] HTML preview updates on DSL change
- [ ] `?code=` parameter works with LZ-compressed base64
- [ ] `#code=` hash parameter works
- [ ] `?data=` JSON parameter works
- [ ] `?url=` fetch parameter works
- [ ] All view modes work (split, editor, viewer, preview)
- [ ] Error handling works for invalid DSL
- [ ] WASM loads successfully
- [ ] No console errors
- [ ] Theme switching works (if implemented)


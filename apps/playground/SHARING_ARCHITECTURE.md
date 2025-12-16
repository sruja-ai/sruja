# Architecture Sharing & URL Stability

## Current Implementation

Currently, sharing uses **LZString compression** in the URL:

- DSL is compressed and embedded in URL: `?code=<compressed_dsl>`
- **Problem**: URL changes every time the architecture mutates
- **Problem**: Long URLs when architecture is large
- **Limitation**: No way to update shared link without creating new URL

## URL Parameters

- `share=<id>`: Share ID (stable, updates don't change URL)
- `code=<compressed>`: Compressed DSL code (for direct sharing)

## Solutions for Stable URLs with Mutations

### Option 1: Backend API (Recommended for Production)

**Pros:**

- ✅ Stable URLs (e.g., `/designer?share=abc123`)
- ✅ Support real-time collaboration
- ✅ Track versions/history
- ✅ Analytics and access control
- ✅ Works across devices

**Cons:**

- ❌ Requires backend infrastructure
- ❌ Not fully local-first (but can be hybrid)

**Implementation:**

```typescript
// Share flow:
1. POST /api/shares { dsl: "..." } → { id: "abc123" }
2. URL becomes: /designer?share=abc123
3. Updates: PATCH /api/shares/abc123 { dsl: "..." }
4. Load: GET /api/shares/abc123 → { dsl: "..." }

// Or with code for first-time sharing:
URL: /designer?share=abc123&code=<compressed_dsl>
```

**Backend Options:**

- **Supabase** (PostgreSQL + Realtime) - Easy setup, free tier
- **Firebase Firestore** - Real-time, easy auth
- **Vercel KV/Upstash** - Simple key-value, serverless
- **Custom Node.js API** - Full control

### Option 2: Hybrid Local-First + Backend (Best of Both Worlds)

**For local-first users:**

- Store locally with unique ID
- Share ID + initial data in URL
- Load from localStorage if ID exists locally

**For sharing across devices:**

- Optional backend sync
- Share ID in URL, backend as fallback
- User chooses: local-only or cloud-synced

**Implementation:**

```typescript
// Generate unique ID on first share
const shareId = crypto.randomUUID();

// Store locally
localStorage.setItem(`share:${shareId}`, dsl);

// URL: /designer?share=abc123 (stable)
// Or with code for first-time sharing: /designer?share=abc123&code=<compressed>
// Load: Check localStorage first, then use code param if not found, then backend
```

### Option 3: IPFS / Decentralized Storage

**Pros:**

- ✅ No backend required
- ✅ Decentralized
- ✅ Content-addressed (hash-based)

**Cons:**

- ❌ More complex setup
- ❌ Slower initial load
- ❌ Still need URL for hash (but hash changes with mutations)

**Implementation:**

- Store DSL on IPFS → Get CID (hash)
- URL: `/designer?ipfs=QmXxx...`
- Mutations create new CID (new URL still)

### Option 4: URL Hash with Version (Compromise)

Keep current LZString but add versioning:

**Implementation:**

```typescript
// Share with version
const shareId = crypto.randomUUID();
const version = 1;
const url = `/designer?share=${shareId}&v=${version}&code=<compressed>`;

// Updates increment version but keep same ID
// URL becomes: ?share=${shareId}&v=${version+1}&code=<new_compressed>
```

**Pros:**

- ✅ Unique ID stays same
- ✅ Version tracking
- ✅ No backend needed

**Cons:**

- ❌ URL still changes (version + data)
- ❌ Can't update existing share without new URL

## Recommendation

For a **local-first, open-source tool** like Excalidraw/draw.io:

### Phase 1: Hybrid Approach (No Backend Required Initially)

1. Generate unique share ID (UUID)
2. Store initial code in URL (compressed) + ID
3. Store updates locally with ID as key
4. URL: `/designer?share=abc123` (ID only) or `/designer?share=abc123&code=<compressed>` (ID + initial code)

**Benefits:**

- Works offline
- No backend dependency
- Can add backend later (backward compatible)

### Phase 2: Optional Backend Sync (User Choice)

1. Add backend API endpoint
2. User chooses: "Share locally" vs "Share via cloud"
3. If cloud: Sync ID → backend
4. If local: Keep current behavior

**Implementation Strategy:**

```typescript
// Share service
class ShareService {
  // Generate unique ID
  generateShareId(): string {
    return crypto.randomUUID();
  }

  // Share locally (current behavior)
  async shareLocal(dsl: string): Promise<string> {
    const id = this.generateShareId();
    const compressed = LZString.compressToBase64(dsl);

    // Store in localStorage
    localStorage.setItem(`share:${id}`, dsl);

    // Return URL with ID (code optional for first load)
    return `${window.location.origin}/designer?share=${id}&code=${encodeURIComponent(compressed)}`;
  }

  // Share via backend (optional)
  async shareCloud(dsl: string): Promise<string> {
    const id = this.generateShareId();
    const response = await fetch("/api/shares", {
      method: "POST",
      body: JSON.stringify({ id, dsl }),
    });
    return `${window.location.origin}/designer?share=${id}`;
  }

  // Load share
  async loadShare(id: string): Promise<string | null> {
    // Try localStorage first
    const local = localStorage.getItem(`share:${id}`);
    if (local) return local;

    // Fallback to backend
    try {
      const response = await fetch(`/api/shares/${id}`);
      const data = await response.json();
      return data.dsl;
    } catch {
      return null;
    }
  }
}
```

## URL Structure Examples

### Direct code sharing:

```
/designer?code=ewogICJhcmNoaXRlY3R1cmUiOiAiRGVtbyIKfQ%3D%3D
```

Compressed DSL code. Changes with every mutation.

### Share ID (stable):

```
/designer?share=abc123-def456-ghi789
```

Stays stable. Code loaded from localStorage or backend.

### Share ID + code (for first-time sharing):

```
/designer?share=abc123&code=ewogICJhcmNoaXRlY3R1cmUiOiAiRGVtbyIKfQ%3D%3D
```

ID stays same, code optional (for first-time loads from another device).

## Backend API Design (If Needed)

### Minimal Implementation (Node.js + SQLite/Postgres)

```typescript
// POST /api/shares
// Create new share
{
  "id": "abc123", // optional, server generates if not provided
  "dsl": "architecture \"Demo\" { ... }"
}
→ { "id": "abc123", "created_at": "2024-..." }

// GET /api/shares/:id
// Get share data
→ { "id": "abc123", "dsl": "...", "updated_at": "2024-..." }

// PATCH /api/shares/:id
// Update share (mutations)
{
  "dsl": "architecture \"Demo\" { ... }"
}
→ { "id": "abc123", "updated_at": "2024-..." }

// DELETE /api/shares/:id (optional)
// Remove share
```

### Simple Backend Options

1. **Vercel Serverless Functions** (Easy, free tier)

   ```typescript
   // api/shares/[id].ts
   import { VercelKV } from "@vercel/kv";

   export default async function handler(req, res) {
     const kv = new VercelKV();
     // Store/retrieve from KV
   }
   ```

2. **Supabase** (PostgreSQL + REST API)
   - Table: `shares(id, dsl, created_at, updated_at)`
   - Auto-generated REST API
   - Free tier: 500MB database

3. **Firebase Firestore**
   - Collection: `shares/{id}`
   - Free tier: 1GB storage

4. **Netlify Functions + FaunaDB** (Serverless)

## Migration Strategy

1. **Keep backward compatibility**: Support both old (`?share=<compressed>`) and new (`?share=<id>`) formats
2. **Gradual rollout**: Start with local-only ID-based sharing
3. **Add backend later**: When users request cloud sync
4. **User choice**: Let users opt-in to cloud sync

## Next Steps

1. Implement ID-based sharing with localStorage
2. Add migration logic to convert old URLs
3. Document new sharing system
4. (Optional) Add backend API for cloud sync
5. Add UI toggle: "Share locally" vs "Share via cloud"

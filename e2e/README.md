# E2E tests (Playwright)

Single E2E test: **book "Show diagram"** – verifies that a ```sruja code block in the book can render a diagram via WASM + Mermaid.

## Prerequisites

1. **Use `serve.sh` so WASM is available** (plain `mdbook serve` wipes the build dir and won’t have WASM):
   ```bash
   just wasm       # or: make wasm
   just book-serve # or: make book-serve
   ```
   Or from repo root: `book/serve.sh`. This builds the book, copies WASM into the output, then serves; a loop re-copies WASM every 0.5s so “Show diagram” works.

2. **Install Node deps and Playwright browsers** (once):
   ```bash
   npm install
   npx playwright install chromium
   ```

## Run

With the book already running at http://localhost:3000:

```bash
npm run e2e
```

Optional: override base URL: `BOOK_BASE_URL=http://127.0.0.1:3000 npm run e2e`

## CI

To run in CI: build book, copy WASM, start `mdbook serve` in the background, wait for port 3000, then run `npm run e2e`. Example (from repo root):

```bash
just wasm  # or: make wasm
cd book && mdbook build && ../book/copy-wasm.sh book && mdbook serve --port 3000 &
npx wait-on http://localhost:3000
npm run e2e
```

# Studio Deployment Modes

## Recommendation

Start with a local, client-side Studio launched via the CLI (`sruja studio`) that serves a static React app. Reuse the same frontend inside a VS Code/JetBrains extension via webview. Consider a cloud service later for organization-wide collaboration and PR automation.

## Options

1. **Local CLI-Launched Studio** (Recommended first)
   - Pure client-side web app served at `http://localhost:<port>`
   - Reads/writes repo files locally; uses user Git credentials
   - Pros: offline, fast iteration, minimal infra, aligns with “No Backend Required”
   - Cons: limited multi-user collaboration; relies on local environment
   - See [Studio Local API Server](go/STUDIO_API.md) for details

2. **Self-Hosted Studio** (For team sharing)
   - Standalone web app deployed on customer infrastructure
   - No Git connection; for sharing previews and designing architectures
   - Pros: easy preview sharing, persistent previews, team collaboration
   - Cons: requires deployment infrastructure
   - See [Self-Hosted Studio](go/SELF_HOSTED_STUDIO.md) for details

3. **IDE Extension** (VS Code/JetBrains)
   - Embed the same Studio frontend in a webview
   - Pros: native UX, file system APIs, SCM integration, commands/shortcuts
   - Cons: duplicate packaging; IDE-specific APIs for FS/PR workflows

4. **Cloud Service** (Future phase)
   - Hosted Studio connects to Git providers, opens PRs
   - Pros: collaboration, auth, centralized governance, review workflows
   - Cons: backend required, secrets management, compliance, cost

## Monorepo Structure

Studio is part of a Turborepo monorepo:

```
apps/studio/          # Studio React application
packages/viewer/      # Viewer library (used by Studio)
packages/shared/      # Shared utilities
```

### GitHub Pages Deployment

Both Learn and Studio are deployed to GitHub Pages:
- **Learn**: `https://sruja.ai/` (root)
- **Studio**: `https://sruja.ai/studio/`

Deployment is automated via `.github/workflows/deploy-pages.yml`:
1. Builds both apps in parallel
2. Combines outputs into `_site/` directory
3. Deploys to GitHub Pages

See [GITHUB_PAGES_SETUP.md](../../GITHUB_PAGES_SETUP.md) for details.

## Flow (Local + IDE)

- `sruja studio` → Starts Go API server + React app (future)
- Studio loads `.sruja` file directly via API (Go reads file, converts to JSON)
- User edits in Studio (changes JSON in memory)
- Studio saves directly to `.sruja` file via API (Go converts JSON to DSL, writes file)
- No export step needed - direct file operations
- Optional: use `gh`/Git APIs to create commits/PRs locally

## CLI (Future)

- `sruja studio` → start local static server and open browser
- `sruja viewer` → open read-only viewer for `.json` or `.sruja.html`

## Next Steps

- Implement `sruja studio` command (Go)
- Package Studio static assets
- Add VS Code extension scaffold embedding Studio
- Explore PR automation via local `gh` or IDE SCM APIs
- Implement GitHub Actions workflow for auto-preview generation
- Add URL parameter support for loading previews (Studio/Viewer)
- See [GitHub PR Integration](go/GITHUB_PR_INTEGRATION.md) for PR preview workflows


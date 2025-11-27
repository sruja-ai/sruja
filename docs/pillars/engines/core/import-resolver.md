# Import Resolver

**Status**: Core Engine  
**Pillar**: All (foundational)

[← Back to Engines](../README.md)

## Overview

The Import Resolver handles loading DSL files from various sources: local files, GitHub repositories, and future registry-based imports.

## Purpose

The resolver supports:
- ✅ Local relative imports (`./file.sruja`)
- ✅ Absolute file imports (`/path/to/file.sruja`)
- ✅ GitHub repo imports (`github.com/org/repo`)
- ✅ GitHub branch/tag/tree imports
- ✅ GitHub direct ZIP archives
- ✅ Registry-based imports (`@org/package`)
- ✅ Caching & reproducibility
- ✅ Version pinning
- ✅ Security & sandboxing

## Import Types

### Local Imports
```sruja
import "./auth.sruja"
import "../shared/contexts.sruja"
import "/abs/path/to/module.sruja"
```

**Detection:**
- Starts with `./` or `../`
- Absolute path (`path.isAbsolute()`)

### GitHub Imports
```sruja
import "github.com/acme/auth-service"
import "github.com/acme/auth-service/tree/dev"
import "github.com/acme/auth-service#v1.2.0"
import "github.com/acme/auth-service/archive/main.zip"
```

**Formats:**
- `github.com/user/repo` - default branch (main)
- `github.com/user/repo/tree/branch` - specific branch
- `github.com/user/repo#tag` - specific tag
- `github.com/user/repo/archive/branch.zip` - direct ZIP

### Registry Imports (Future)
```sruja
import "@acme/identity"
import "@company/bounded-context/payments"
import "@acme/identity@1.9.0"
```

## Directory Layout

```
.architecture/
  imports/                    # cached modules
    github.com/acme/auth-service/
       v1.2.0/
          architecture.sruja
       dev/
          architecture.sruja

  imports.lock                # resolved versions (for deterministic builds)
```

## Architecture

```
importResolver.ts
  ├── isLocalImport()
  ├── isGithubImport()
  ├── isRegistryImport()
  ├── resolveLocal()
  ├── resolveGithub()
  ├── resolveRegistry()
  ├── downloadZip()
  ├── extractZip()
  └── cacheManager()
```

## Implementation

### Resolver Entry Point

```typescript
export async function resolveImportPath(
  specifier: string,
  cwd: string,
  projectRoot: string
): Promise<ResolvedImport> {

  if (isLocalImport(specifier))
    return resolveLocal(specifier, cwd);

  if (isGithubImport(specifier))
    return await resolveGithub(specifier, projectRoot);

  if (isRegistryImport(specifier))
    return await resolveRegistry(specifier, projectRoot);

  throw new Error(`Unsupported import: ${specifier}`);
}
```

### Local Import Resolution

```typescript
function isLocalImport(s: string) {
  return s.startsWith("./") || s.startsWith("../") || path.isAbsolute(s);
}

function resolveLocal(specifier: string, cwd: string): ResolvedImport {
  const fullPath = path.resolve(cwd, specifier);

  if (!fs.existsSync(fullPath))
    throw new Error(`Local import not found: ${fullPath}`);

  return {
    source: "local",
    path: fullPath,
    version: "local",
    cachedPath: fullPath, // no caching needed
  };
}
```

### GitHub Import Resolution

#### Parse GitHub Spec

```typescript
function parseGithub(specifier: string): GithubSpec {
  const re = /^github\.com\/([^/]+)\/([^/#]+)(?:\/tree\/([^#]+))?(?:#(.+))?/;
  const m = re.exec(specifier);
  if (!m) throw new Error("Invalid GitHub import format");

  const [, user, repo, branchOrTag, tag] = m;
  return {
    user,
    repo,
    ref: tag || branchOrTag || "main"
  };
}
```

#### Build GitHub ZIP URL

```typescript
function buildGithubZipUrl({ user, repo, ref }: GithubSpec) {
  if (isTag(ref)) {
    return `https://github.com/${user}/${repo}/archive/refs/tags/${ref}.zip`;
  }
  return `https://github.com/${user}/${repo}/archive/refs/heads/${ref}.zip`;
}
```

#### Download and Cache

```typescript
async function resolveGithub(specifier: string, projectRoot: string) {
  const spec = parseGithub(specifier);
  const zipUrl = buildGithubZipUrl(spec);

  const cacheDir = path.join(projectRoot, ".architecture/imports/github.com", spec.user, spec.repo, spec.ref);
  const dslPath = path.join(cacheDir, "architecture.sruja");

  if (fs.existsSync(dslPath))
    return { source: "github", path: dslPath, cachedPath: dslPath, version: spec.ref };

  await fs.mkdir(cacheDir, { recursive: true });

  const zipPath = path.join(cacheDir, "source.zip");
  await downloadZip(zipUrl, zipPath);

  await extractZip(zipPath, cacheDir);

  // find architecture.sruja
  const file = findArchitectureFile(cacheDir);

  // lock file update
  updateLockFile(specifier, spec.ref, file);

  return {
    source: "github",
    version: spec.ref,
    path: file,
    cachedPath: file
  };
}
```

### ZIP Download

```typescript
async function downloadZip(url: string, dest: string) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed ZIP download: ${res.status}`);
  const buf = Buffer.from(await res.arrayBuffer());
  await fs.writeFile(dest, buf);
}
```

### ZIP Extraction

Uses JSZip (works in Bun):

```typescript
async function extractZip(zipFile: string, dest: string) {
  const data = await fs.readFile(zipFile);
  const zip = await JSZip.loadAsync(data);

  for (const name of Object.keys(zip.files)) {
    const file = zip.files[name];
    const outPath = path.join(dest, name);

    if (file.dir) {
      await fs.mkdir(outPath, { recursive: true });
    } else {
      const content = await file.async("nodebuffer");
      await fs.mkdir(path.dirname(outPath), { recursive: true });
      await fs.writeFile(outPath, content);
    }
  }
}
```

### Registry Import (Future)

Syntax:
```sruja
import "@acme/identity@1.4.0"
import "@acme/payments"
```

Design:
```json
// .architecture/registry.json
{
  "@acme/identity": "github.com/acme/identity-service",
  "@acme/payments": "github.com/acme/payments-service"
}
```

Implementation:
```typescript
async function resolveRegistry(specifier: string, projectRoot: string) {
  const { packageName, version } = parseRegistrySpecifier(specifier);
  const registry = loadRegistry(projectRoot);
  const githubUrl = registry[packageName];
  
  if (!githubUrl) throw new Error(`Package not found: ${packageName}`);
  
  return await resolveGithub(`${githubUrl}#${version}`, projectRoot);
}
```

## Lock File

Stores resolved versions for deterministic builds:

```json
// .architecture/imports.lock
{
  "github.com/acme/auth-service": {
    "version": "v1.2.0",
    "commit": "abc123...",
    "cachedPath": ".architecture/imports/github.com/acme/auth-service/v1.2.0"
  }
}
```

## Caching Strategy

- **Local imports**: No caching (always read from disk)
- **GitHub imports**: Cached by `user/repo/ref` in `.architecture/imports/`
- **Registry imports**: Resolved to GitHub, then cached

## Security Considerations

- ✅ Sandboxed extraction (no code execution)
- ✅ Version pinning prevents supply chain attacks
- ✅ Lock file ensures reproducible builds
- ✅ No network access after initial download

## Integration

Used by:
- **Model Composition Engine** - Resolves imports during composition
- **CLI** - `sruja compile` command
- **LSP** - Import resolution for diagnostics

## Implementation Status

✅ Architecture designed  
✅ Local import resolution specified  
✅ GitHub import resolution specified  
✅ Caching strategy defined  
📋 Registry imports planned  
📋 Implementation in progress

---

*The Import Resolver enables modular architecture composition across projects and repositories.*

# Express.js Architecture Comparison: 3 Levels

**Test:** Fair comparison - same codebase, different approaches
1. **NO SKILL**: Mermaid diagram (natural AI output)
2. **WITH SKILL (Standard)**: Sruja DSL - key relationships
3. **WITH SKILL (Deep)**: Sruja DSL - internal patterns

## Metrics

| Metric | Mermaid | Standard | Deep |
|--------|---------|----------|------|
| Lines | 48 | 107 | 305 |
| Elements | 17 | 19 | 55 |
| Relationships | 14 | 19 | 46 |
| Requirements | 0 | 0 | 5 |
| ADRs | 0 | 0 | 5 |

## What Each Captured

### Internal Patterns (the missing pieces)

| Pattern | Mermaid | Standard | Deep |
|---------|---------|----------|------|
| **Layer abstraction** | ✗ | ✗ | ✓ `router.layer` wraps middleware |
| **Error handling** | ✗ | ✗ | ✓ `middleware.errorHandler` uses `finalhandler` |
| **Mount pattern** | ✗ | ✗ | ✓ `app.mountHandler` + `webApp.subApp` |
| **View caching** | ✗ | ✗ | ✓ `view.cache` enabled in production |
| **Lazy router init** | ✗ | ✗ | ✓ `app -> router "lazily initializes"` |

### Components Captured

| Category | Mermaid | Standard | Deep |
|----------|---------|----------|------|
| Core (app, router, req, res) | ✓ | ✓ | ✓ |
| Middleware | ✓ | ✓ | ✓ (with error handler) |
| View system | ✓ | ✓ | ✓ (with cache, resolver) |
| Config | ✗ | ✓ | ✓ (with locals) |
| NPM packages | 3 | 3 | 6 |
| Internal components | 0 | 0 | 15 |
| Requirements | 0 | 0 | 5 |
| ADRs | 0 | 0 | 5 |

## Deep Scope Specifics

### 1. Layer Abstraction
```sruja
router = container "Router" {
  layer = component "Layer" {
    description "Wraps each middleware/handler with path matching"
  }
  stack = component "Stack" {
    description "Ordered array of Layer instances"
  }
  router.layer -> router.stack "registered in"
}
```

### 2. Error Handling Path
```sruja
middleware = container "Middleware Pipeline" {
  errorHandler = component "Error Handler" {
    description "Final handler using finalhandler package"
  }
}
express.middleware.errorHandler -> npm.finalhandler "uses for error handling"
express.middleware.errorHandler -> express.app.errorLogger "logs errors via"
```

### 3. Mount Pattern
```sruja
app = container "Application" {
  mountHandler = component "Mount Handler" {
    description "Handles sub-app mounting via app.use(subapp)"
  }
}
webApp = system "Web Application" {
  mainApp -> subApp "mounts via app.use() at path"
  subApp -> express "inherits settings via on('mount')"
}
```

### 4. View Caching (Env-Specific)
```sruja
view = container "View System" {
  cache = component "View Cache" {
    description "Template cache. Enabled when NODE_ENV=production"
  }
}
view.cache -> express.app.config "reads 'view cache' setting from"
```

### 5. ADRs Captured
```sruja
ADR003 = adr "Layer abstraction for middleware" {
  status "Accepted"
  context "Middleware needs path matching and ordered execution"
  decision "Wrap each middleware in Layer class with path and handle()"
  consequences "Clean stack management, enables route-specific middleware"
}

ADR004 = adr "Mount pattern for sub-apps" {
  status "Accepted"
  decision "Allow app.use(subapp) with mountpath, parent, 'mount' event"
  consequences "Enables modular apps, settings inheritance"
}

ADR005 = adr "View cache enabled in production" {
  status "Accepted"
  decision "Auto-enable view cache when NODE_ENV=production"
  consequences "Faster rendering in production, must restart for changes"
}
```

## Verdict

| Use Case | Best Choice |
|----------|-------------|
| Quick overview | Mermaid |
| Architecture documentation | Standard |
| **Architecture governance** | **Deep** |
| Onboarding | Standard + Deep |
| Drift detection | Deep |
| Pattern documentation | Deep only |

**Deep scope captures 3x more architectural insight** by documenting:
- **How** components interact (not just that they do)
- **Why** decisions were made (ADRs)
- **When** behavior changes (env-specific caching)
- **What** internal abstractions exist (Layer, Stack)

## Skill Improvement Impact

| Before skill update | After skill update (Deep) |
|--------------------|---------------------------|
| Layer abstraction: ✗ | ✓ Detected via `grep "class.*Layer"` |
| Error handling: ✗ | ✓ Detected via `grep "finalhandler"` |
| Mount pattern: ✗ | ✓ Detected via `grep "mount|parent"` |
| View caching: ✗ | ✓ Detected via `grep "NODE_ENV|production"` |

The skill's new **Internal Abstractions** section guides agents to find these patterns.

## Files

```
comparison-express/
├── COMPARISON.md
├── no-skill/
│   └── architecture.mmd          # 48 lines, 17 elements
├── with-skill/
│   └── architecture.sruja        # 107 lines, 19 elements
└── with-skill-deep/
    └── architecture.sruja        # 305 lines, 55 elements, 5 ADRs
```

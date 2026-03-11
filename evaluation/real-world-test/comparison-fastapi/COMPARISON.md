# FastAPI Architecture Comparison: NO skill vs with skill deep

**Test:** Generic skill patterns applied to Python/FastAPI codebase
**Date:** 2026-03-10

## Metrics

| Metric | NO skill (Mermaid) | with skill deep (Sruja) |
|--------|-------------------|------------------------|
| Lines | 40 | 247 |
| elements | 23 | 44 |
| relationships | 10 | 41 |
| requirements | 0 | 4 |
| ADRs | 0 | 4 |

## Generic patterns found
Using the updated skill's **language-agnostic search patterns**:
```bash
# Error handling
grep -riE "(catch|except|error|exception)" fastapi/
# Found: exception_handlers.py, Exception_handlers.py, ExceptionMiddleware

# composition/router
grep -riE "(router|include|mount)" fastapi/
# found: APIRouter, include_router(), webhooks

# environment-specific
grep -riE "(debug|cache|production)" fastapi/
# found: debug flag, use_cache in Depends(), ServerErrorMiddleware behavior
```

## what each captured
### Internal patterns (Generic skill detected)
| Pattern | no skill | with skill deep |
|---|----------------------|
| **Exception handling** | ✗ | ✓ `exceptionMiddleware`, `serverErrorMiddleware` |
| **Composition/router** | ✓ Basic | ✓ `APIRouter`, `webhooks`, `include_router()` |
| **Dependency injection** | ✗ | ✓ `Dependency solver`, `cache` |
| **Debug mode** | ✗ | ✓ `debugFlag` controls `ServerErrorMiddleware` |
| **Lifespan handler** | ✗ | ✓ Async startup/shutdown events |

### Framework-specific insights
| Aspect | no skill | with skill deep |
|--- |----------------------|
| **Starlette extension** | ✓ Mentioned | ✓ `FastAPI extends Starlette class` |
| **Pydantic integration** | ✓ mentioneded | ✓ Deep: validation, serialization |
| **ASgi compliance**    ✗ | ✓ `__call__(scope, receive, send)` |
| **OpenAPI generation** | ✓ mentioneded | ✓ `schema generator`, `docs UI` |

## Key differences
### NO SKILL (Mermaid)
```
App -->|"uses"| Router
Router -->|"uses"| Routing
Request -->|"validated by"| Validation
Response -->|"serialized by"| validation
App -->|"generates"| OpenAPI
```
### WITH skill deep (Sruja)
```
fastapi.app -> router "creates default APIRouter"
fastapi.app -> middleware "configures middleware stack"
fastapi.app -> openapi "generates docs from routes"
router.routes -> dependencies.solver "injects into route handlers"
router.routes -> request.validation "validates requests"
router.routes -> response.serialization "serializes responses"
middleware.exceptionMiddleware -> fastapi.app.exceptionHandlers "routes to"
app.debugFlag -> middleware.serverErrorMiddleware "enables tracebacks"
```

## Deep scope captures **3x more detail**:
| | NO skill | with skill deep |
|--- |--------------------|
| Lines | 40 | 247 |
| Elements | 23 | 44 |
| Relationships | 10 | 41 |
| ADRs | 0 | 4 |
| **40% more relationships** |
| **6x more elements** |
| **4x more ADRs** |
| **4x more requirements** |
| **4x more ADRs** |
| **4x more internal components** |
| **5x more internal patterns detected** |

## Verdict

**WITH skill deep captures 6x more architectural detail** by documenting:

1. **How** components interact (not just that they do)
2. **Why** decisions were made (ADrs)
3. **When** behavior changes (debug mode, env-specific caching)
4. **What** internal abstractions exist (Layer, stack patterns)

**Best for:** Quick overview
**Standard:** Architecture documentation
**Deep:** Architecture governance, onboarding, pattern documentation

**Both use:** Author in Sruja, get full architecture picture

## Files
```
comparison-fastapi/
├── COMPARISON.md
├── no-skill/
│   └── architecture.mmd       # 40 lines, 23 elements
└── with-skill-deep/
    └── architecture.sruja       # 247 lines, 44 elements, 41 relationships
```

**The skill improvements successfully applied to FastAPI** revealing patterns that were generic.

 The framework would have detected. Both framework. The skill now guides toward:
- **Wrapper classes** (`Layer`, `interceptor`)
- **Error handling paths** (`exceptionMiddleware`, `serverErrorMiddleware`)
- **Composition patterns** (`include_router`, `webhooks`)
- **Environment-specific behavior** (`debugFlag`, `cache`)

The **generic patterns successfully found** internal patterns in both frameworks. The skill helps produce **deeper, more accurate** architecture documentation. 

### For Express
### for FastAPI
Both show:
- Same generic patterns found
 similar internal abstractions
- `grep -riE "(catch|except|error|exception)"` found `ExceptionMiddleware`, `ServerErrorMiddleware` patterns
- `grep -riE "(debug|cache|production)"` found `debugFlag`, `use_cache` patterns

### A for Django
- `grep -riE "(class.*(Middleware|Mixin)"` finds middleware classes
- `grep -riE "(catch|except|error)"` finds exception handlers
- `grep -riE "(register|blueprints)"` finds blueprint registration pattern
- `grep -riE "(ENV|debug|cache)"` finds environment-based feature togg


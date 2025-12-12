# Sruja DSL Feature Coverage Analysis

## Summary
This document compares DSL features available in the language parser vs. what's documented in docs and courses.

## ✅ Well Documented Features

### Core Elements
- ✅ `person` - Documented in `/docs/concepts/person.md`
- ✅ `system` - Documented in `/docs/concepts/system.md`
- ✅ `container` - Documented in `/docs/concepts/container.md`
- ✅ `component` - Documented in `/docs/concepts/component.md`
- ✅ `datastore` - Documented in `/docs/concepts/datastore.md`
- ✅ `queue` - Documented in `/docs/concepts/queue.md`

### Relations
- ✅ Relations (`->`) - Documented in `/docs/concepts/relations.md`

### Basic Metadata
- ✅ `metadata` blocks - Documented in `/docs/concepts/metadata-and-tags.md`
- ✅ `tags` arrays - Documented in `/docs/concepts/metadata-and-tags.md`
- ✅ `technology` field - Documented in `/docs/concepts/metadata-and-tags.md`
- ✅ `description` field - Used throughout examples

### Advanced Features
- ✅ `scenario` / `story` - Documented in `/docs/concepts/scenario.md`
- ✅ `deployment` / `node` / `containerInstance` - Documented in `/docs/concepts/deployment.md`
- ✅ `requirement` - Covered in courses (system-design-101)
- ✅ `adr` - Documented in `/docs/concepts/adr.md`

### Validation
- ✅ Validation rules - Documented in `/docs/concepts/validation.md`
- ✅ Layering - Documented in `/docs/concepts/layering.md`

## ⚠️ Partially Documented Features

### Requirements
- ⚠️ `requirement` - Used in courses but no dedicated concept doc
- ⚠️ Requirement types: `functional`, `performance`, `security`, `constraint` - Only shown in examples

### Scenarios
- ⚠️ `flow` keyword (DFD-style flows) - Not explicitly documented, only `scenario` is
- ⚠️ Scenario step properties (tags, order) - Not documented

## ❌ Missing Documentation

### Advanced Metadata & Properties
- ❌ `properties` blocks - Available in AST, not documented
- ❌ `style` blocks - Available in AST, not documented
- ❌ `overview` blocks - Available in AST, not documented

### Scaling & Performance
- ❌ `scale` blocks (`min`, `max`, `metric`) - Available in AST, mentioned in course lesson-2.md but not in concept docs
- ✅ `slo` blocks (availability, latency, errorRate, throughput) - Documented in `/apps/website/src/content/docs/concepts/slo.md`

### Contracts & Constraints
- ❌ `contracts` blocks (`api`, `event`, `data`) - Available in AST, not documented
- ❌ `constraints` blocks - Available in AST, not documented
- ❌ `conventions` blocks - Available in AST, not documented

### Libraries & Policies
- ❌ `library` keyword - Available in AST, not documented
- ❌ `sharedArtifact` keyword - Available in AST, not documented
- ✅ `policy` keyword - Implemented; style-guide notes policy is supported (rule blocks not yet implemented)

### Views & Customization
- ❌ `views` blocks - Available in AST, not documented
- ❌ View expressions (`include`, `exclude`) - Available in AST, not documented
- ❌ View styles - Available in AST, not documented

### Change Management
- ❌ `change` blocks - Available in AST, not documented
- ❌ `snapshot` blocks - Available in AST, not documented

### Imports
- ❌ `import` keyword - Available in AST, not documented
- ❌ Import aliases (`import "file.sruja" as alias`) - Available in AST, not documented

### Component Features
- ❌ `behavior` blocks - Available in AST, not documented
- ❌ `depends_on` field - Available in AST, not documented
- ❌ `version` field - Available in AST, not documented

### External Elements
- ❌ `external` keyword - Not found in AST (may be deprecated or use tags instead)
- ⚠️ External services pattern - Style guide says to use `tags ["external"]` instead

## Course Coverage

### system-design-101
- ✅ Basic elements (person, system, container, datastore)
- ✅ Relations
- ✅ Requirements (lesson-1.md)
- ✅ Scale blocks (lesson-2.md)
- ✅ Scenarios (lesson-5.md)
- ⚠️ SLO blocks - Not covered
- ⚠️ Contracts - Not covered
- ⚠️ Policies - Mentioned but marked as deferred

### ecommerce-platform
- ✅ Basic architecture modeling
- ✅ Deployment (module-5-ops)
- ✅ ADRs (module-3-architecture-tech)
- ⚠️ Advanced features - Limited coverage

## Recommendations

### High Priority (Core Features)
1. **Document `scale` blocks** - Used in courses, should be in concept docs
2. **Document `slo` blocks** - Important for production systems
3. **Document `import`** - Essential for multi-file architectures
4. **Document `flow` vs `scenario`** - Clarify the difference

### Medium Priority (Advanced Features)
5. **Document `contracts` blocks** - API/event/data contracts
6. **Document `library` and `sharedArtifact`** - Reusability features
7. **Document `policy`** - Actually implemented, not deprecated
8. **Document `properties` and `style` blocks** - Customization

### Low Priority (Specialized Features)
9. **Document `views` blocks** - Advanced diagram customization
10. **Document `change` and `snapshot` blocks** - Version control features
11. **Document `overview` blocks** - Architecture documentation
12. **Document `behavior` blocks** - Component behavior specification

## Statistics

- **Total DSL Features**: ~35
- **Well Documented**: ~15 (43%)
- **Partially Documented**: ~3 (9%)
- **Missing Documentation**: ~17 (48%)

## Notes

- `flow` was previously listed as deprecated in the style guide; it is implemented as an alias for `scenario` and accepted for legacy content (prefer `scenario`).
- The `external` keyword doesn't appear in AST - external services should use `tags ["external"]` pattern.
- Many advanced features exist but aren't discoverable without reading the source code.

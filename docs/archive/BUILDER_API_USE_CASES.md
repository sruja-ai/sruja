# When Do We Need a Builder API in the Backend?

## TL;DR

**You DON'T need a Builder API if:**
- Users primarily write DSL files manually
- You only parse/validate/export existing DSL
- Your use cases are read-only

**You DO need a Builder API if:**
- You need to generate models programmatically
- You're converting from other formats (JSON, YAML, OpenAPI, etc.)
- You're writing complex tests that create models
- You're building code generation tools
- You're creating CLI tools that generate starter templates

## Current State in Sruja

Looking at the codebase, I found **781 instances** where DSL strings are used in tests and code. This suggests:

1. **Tests extensively use DSL strings** - Every test parses DSL strings
2. **CLI `init` command uses string templates** - Creates DSL files via string concatenation
3. **No programmatic model creation** - Everything goes through the parser

## Use Cases Where Builder API Would Help

### 1. **Testing** (High Value)

**Current approach:**
```go
func TestValidReferences_PersonContainer(t *testing.T) {
    dsl := `
model {
    Customer = person "Customer"
    Order = system "Order System" {
      API = container "Order API"
    }
    Customer -> API "Uses"
}
`
    program := parse(t, dsl)
    // ... test logic
}
```

**With Builder API:**
```go
func TestValidReferences_PersonContainer(t *testing.T) {
    builder := NewBuilder(Specification{
        Elements: []string{"person", "system", "container"},
    })
    
    program := builder.
        Model(func(m *ModelBuilder) {
            m.Person("Customer", "Customer")
            m.System("Order", "Order System").
                With(func(n *NestedBuilder) {
                    n.Container("API", "Order API")
                })
            m.Rel("Customer", "Order.API", "Uses")
        }).
        Build()
    
    // ... test logic
}
```

**Benefits:**
- Type-safe (compile-time errors)
- No string parsing overhead in tests
- Easier to modify programmatically
- Better IDE autocomplete

**Verdict:** ✅ **Worth it** - Tests are written frequently, and this would improve developer experience

---

### 2. **CLI `init` Command** (Medium Value)

**Current approach** (`cmd/sruja/init.go`):
```go
mainContent := `model {
	// Define your system here
	system MySystem "My System" {
		description "A new Sruja system"
	}
}
`
```

**With Builder API:**
```go
builder := NewBuilder(DefaultSpecification())
program := builder.
    Model(func(m *ModelBuilder) {
        m.System("MySystem", "My System").
            With(func(n *NestedBuilder) {
                n.Description("A new Sruja system")
            })
    }).
    Build()

dsl := ExportToDSL(program)
```

**Benefits:**
- Type-safe template generation
- Can generate more complex templates
- Easier to maintain

**Verdict:** ⚠️ **Maybe** - Current string template works fine for simple cases

---

### 3. **Import/Export from Other Formats** (High Value)

**Current state** (`pkg/import/json/json.go`):
- Methods are deprecated: `ToArchitecture` and `ToDSL` return errors
- No active implementation for converting JSON → Sruja model

**With Builder API:**
```go
// Convert OpenAPI spec to Sruja model
func OpenAPIToSruja(openapi *openapi3.T) (*language.Program, error) {
    builder := NewBuilder(Specification{
        Elements: []string{"system", "container", "component"},
    })
    
    for path, pathItem := range openapi.Paths {
        builder.Model(func(m *ModelBuilder) {
            m.System("API", "REST API").
                With(func(n *NestedBuilder) {
                    for method, operation := range pathItem.Operations() {
                        n.Component(
                            fmt.Sprintf("%s_%s", method, path),
                            operation.Summary,
                        )
                    }
                })
        })
    }
    
    return builder.Build(), nil
}
```

**Use cases:**
- Import from OpenAPI/Swagger
- Import from Terraform (infrastructure as code)
- Import from Kubernetes manifests
- Import from AWS CloudFormation
- Convert from other architecture tools (Structurizr, PlantUML, etc.)

**Verdict:** ✅ **Worth it** - Enables powerful import/export capabilities

---

### 4. **Code Generation** (High Value)

**Example: Generate architecture from code analysis:**
```go
// Analyze Go codebase and generate architecture model
func GenerateFromCodebase(rootPath string) (*language.Program, error) {
    builder := NewBuilder(DefaultSpecification())
    
    packages := analyzeGoPackages(rootPath)
    
    builder.Model(func(m *ModelBuilder) {
        for _, pkg := range packages {
            m.System(pkg.Name, pkg.Name).
                With(func(n *NestedBuilder) {
                    for _, service := range pkg.Services {
                        n.Container(service.Name, service.Name)
                    }
                })
        }
    })
    
    return builder.Build(), nil
}
```

**Use cases:**
- Generate architecture docs from code
- Reverse-engineer architecture from codebase
- Generate models from database schemas
- Generate models from API documentation

**Verdict:** ✅ **Worth it** - Enables powerful automation

---

### 5. **API Endpoints** (Low Value - Not Currently Needed)

**Hypothetical: REST API to create models**
```go
// POST /api/models
func CreateModelHandler(w http.ResponseWriter, r *http.Request) {
    var req CreateModelRequest
    json.NewDecoder(r.Body).Decode(&req)
    
    builder := NewBuilder(Specification{
        Elements: req.Specification.Elements,
    })
    
    program := builder.
        Model(func(m *ModelBuilder) {
            for _, elem := range req.Elements {
                m.AddElement(elem)
            }
        }).
        Build()
    
    // Save or return model
}
```

**Verdict:** ❌ **Not needed** - Sruja doesn't have a REST API, and DSL-first approach is better for this use case

---

### 6. **Migrations** (Medium Value)

**Example: Migrate old format to new format**
```go
func MigrateOldFormat(old *OldArchitecture) (*language.Program, error) {
    builder := NewBuilder(DefaultSpecification())
    
    builder.Model(func(m *ModelBuilder) {
        for _, sys := range old.Systems {
            m.System(sys.ID, sys.Name).
                With(func(n *NestedBuilder) {
                    for _, cont := range sys.Containers {
                        n.Container(cont.ID, cont.Name)
                    }
                })
        }
    })
    
    return builder.Build(), nil
}
```

**Verdict:** ⚠️ **Maybe** - Only needed if you're migrating from old formats

---

## Cost-Benefit Analysis

### Implementation Cost
- **High**: ~1000-2000 lines of Go code
- Need to maintain parity with DSL features
- Need comprehensive tests
- Need documentation

### Benefits
- **Testing**: Significant improvement in test DX
- **Import/Export**: Enables powerful integrations
- **Code Generation**: Enables automation
- **Type Safety**: Compile-time validation

### When It's NOT Worth It
- If you only parse/validate DSL (read-only)
- If users always write DSL manually
- If you don't need programmatic generation
- If the implementation cost outweighs benefits

---

## Recommendation

**For Sruja specifically:**

1. **Short-term**: Keep DSL-first approach
   - Current approach works well
   - Users write DSL files
   - No urgent need for Builder API

2. **Medium-term**: Consider Builder API if:
   - You want to add import from other formats (OpenAPI, Terraform, etc.)
   - Tests become too verbose with DSL strings
   - You want to build code generation tools

3. **Long-term**: Builder API becomes valuable when:
   - You have multiple import sources
   - You're building automation tools
   - You want better test ergonomics

**Bottom line**: You don't **need** it now, but it would be **useful** for:
- Better test ergonomics
- Import/export capabilities
- Code generation tools

The DSL-first approach is perfectly valid and works well for most use cases.

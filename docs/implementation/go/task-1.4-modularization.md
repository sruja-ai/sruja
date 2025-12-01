# Task 1.4: Modularization Command (Split Large Files)

**Priority**: 🟡 High (Developer Experience)
**Technology**: Go
**Estimated Time**: 3-5 days
**Dependencies**: Task 1.1 (needs JSON), Task 1.2 (needs JSON-to-AST)

## Files to Create

* `pkg/refactor/modularize.go` - Modularization logic
* `pkg/refactor/splitter.go` - File splitting strategies
* `cmd/sruja/modularize.go` - CLI command

## Command

```bash
# Split by system (each system becomes its own file)
sruja modularize --strategy=system <input.sruja> <output-dir>

# Split by domain (each domain becomes its own file)
sruja modularize --strategy=domain <input.sruja> <output-dir>

# Split by feature (custom grouping)
sruja modularize --strategy=feature <input.sruja> <output-dir>

# Split with shared/common elements
sruja modularize --strategy=system --shared <input.sruja> <output-dir>
```

## Strategies

1. **By System** (`--strategy=system`)
   * Each system → separate file
   * Shared elements (persons, shared artifacts, etc.) → `shared.sruja`
   * Main file imports all system files
   * Relations preserved (cross-file references)

2. **By Domain** (`--strategy=domain`)
   * Each domain → separate file
   * Shared elements → `shared.sruja`
   * Main file imports all domain files

3. **By Feature** (`--strategy=feature`)
   * Groups related systems/domains together
   * Uses metadata/tags to determine grouping
   * Custom grouping rules

## Example Output

**Input:** `monolith.sruja`

```sruja
architecture "E-commerce Platform" {
  person Customer "Customer"
  person Admin "Admin"
  
  system Shop "Shop System" {
    container WebApp "Web App" {}
    container API "API" {}
    datastore DB "Database" {}
  }
  
  system Payment "Payment System" {
    container Gateway "Payment Gateway" {}
  }
  
  Customer -> Shop "Uses"
  Shop -> Payment "Processes payment"
}
```

**Output after** `sruja modularize --strategy=system monolith.sruja ./modules`:

**`modules/shared.sruja`**
```sruja
person Customer "Customer"
person Admin "Admin"
```

**`modules/Shop.sruja`**
```sruja
import "shared.sruja"

system Shop "Shop System" {
  container WebApp "Web App" {}
  container API "API" {}
  datastore DB "Database" {}
}
```

**`modules/main.sruja`**
```sruja
import "shared.sruja"
import "Shop.sruja"
import "Payment.sruja"

architecture "E-commerce Platform" {
  Customer -> Shop "Uses"
  Shop -> Payment "Processes payment"
}
```

## Acceptance Criteria

* [ ] Can split by system
* [ ] Can split by domain
* [ ] Shared elements extracted correctly
* [ ] Imports generated correctly
* [ ] Cross-file relations preserved
* [ ] Generated files are valid Sruja DSL
* [ ] Can combine split files back to original (round-trip)

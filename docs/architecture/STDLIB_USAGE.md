# Using sruja.ai/stdlib in Architecture Files

## Should We Use stdlib?

**Yes, we should use `sruja.ai/stdlib`** for architecture files that describe Sruja itself. Here's why:

### Benefits

1. **DRY Principle**: Don't repeat standard element definitions
2. **Consistency**: Ensures we use the same element definitions as the rest of the codebase
3. **Maintainability**: If stdlib changes, our files automatically use the updated definitions
4. **Best Practice**: Demonstrates proper use of stdlib to users

### Current Situation

The stdlib (`pkg/stdlib/core.sruja`) defines:
- Standard C4 elements: `person`, `system`, `container`, `component`, `database`, `queue`
- Standard tags: `external`, `deprecated`

## How to Use stdlib

### Option 1: Import and Extend (Recommended)

```sruja
import { * } from 'sruja.ai/stdlib'

model {
  // Your model elements
  // Standard elements are available from stdlib
}
```

### Option 2: Explicit Import

```sruja
import { person, system, container, component } from 'sruja.ai/stdlib'

model {
  // Use imported elements
}
```

### Option 3: Keep Current (Not Recommended)

```sruja
specification {
  element person
  element system
  // ... redefining what stdlib already provides
}
```

## Recommendation

**Use Option 1** - Import everything from stdlib since we're using standard C4 elements.

### Updated Architecture Files

Both `sruja-platform.sruja` and `sruja-development-workflow.sruja` should:

1. **Remove** the `specification { }` block (or keep minimal if extending)
2. **Add** `import { * } from 'sruja.ai/stdlib'` at the top
3. **Keep** the `model { }` and `views { }` blocks as-is

## Example

### Before (Current)
```sruja
specification {
  element person
  element system
  element container
  element component
}

model {
  // ...
}
```

### After (With stdlib)
```sruja
import { * } from 'sruja.ai/stdlib'

model {
  // Standard elements available from stdlib
  // No need to redefine them
}
```

## Notes

- The stdlib is embedded in the Go binary, so it works in WASM
- Imports are resolved automatically by the parser
- If you need custom elements beyond stdlib, you can still define them in a `specification` block
- The stdlib provides the standard C4 model elements, which is what we need

## Implementation

Update the architecture files to use stdlib imports instead of manually defining specifications.


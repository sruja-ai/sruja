# Views and Implied Relationships Implementation

## Overview

This document describes the implementation of two key enhancements inspired by Structurizr DSL:

1. **Implied Relationships** - Automatically infer parent relationships (DRY principle)
2. **Optional Views Block** - Customize diagram views (optional, C4 views remain automatic)

## Implied Relationships

### What It Does

Automatically infers parent relationships when child relationships exist.

**Example:**
```sruja
User -> API.WebApp "Uses"
// Automatically infers: User -> API
```

### Implementation

- **Location:** `pkg/language/ast_postprocess.go`
- **Method:** `inferImpliedRelationships()`
- **When:** Called during `PostProcess()` after all relationships are collected
- **Logic:** 
  - If `A -> B.C` exists (where B.C is nested), infer `A -> B`
  - Only infers when "From" is not nested within parent of "To"
  - Prevents duplicate relationships

### Tests

- **File:** `pkg/language/ast_postprocess_implied_test.go`
- **Coverage:** Simple inference, multiple relationships, duplicate prevention

## Optional Views Block

### What It Does

Allows customization of diagram views while keeping C4 standard views automatic by default.

**Key Design Decision:** Views are **optional** - if omitted, standard C4 views are automatically generated. This aligns with Sruja's "start simple" philosophy.

### Syntax

```sruja
architecture "Example" {
    // Model definition...
    
    // OPTIONAL: Only define if you need customization
    views {
        container Shop "API Focus" {
            include Shop.API Shop.DB
            exclude Shop.WebApp
            autolayout lr
        }
        
        styles {
            element "Database" {
                shape "cylinder"
                color "#ff0000"
            }
        }
    }
}
```

### Implementation

#### AST Structures

- **File:** `pkg/language/ast_views.go`
- **Types:**
  - `ViewBlock` - Container for views and styles
  - `View` - Individual view definition
  - `ViewExpression` - Include/exclude expressions
  - `StylesBlock` - Styling definitions
  - `ElementStyle` - Style for elements by tag

#### Parser Support

- **File:** `pkg/language/parser.go`
- **Changes:**
  - Added `Wildcard` token for `include *` expressions
  - Added `views` keyword support

#### Post-Processing

- **File:** `pkg/language/ast_postprocess.go`
- **Changes:**
  - Added `Views` field to `Architecture`
  - Views block is stored during post-processing

#### Export Integration

- **File:** `pkg/export/views/views.go`
- **Functions:**
  - `ApplyViewExpressions()` - Filter elements based on view expressions
  - `ApplyStyles()` - Apply styles from views block
  - `FindViewByName()` - Find view by name
  - `GetAutolayoutDirection()` - Get layout direction from view

#### Markdown Export

- **File:** `pkg/export/markdown/markdown.go`
- **Changes:**
  - Checks for custom views before generating standard C4 views
  - Uses custom views if defined, otherwise falls back to automatic views

### Tests

- **File:** `pkg/language/ast_views_test.go`
- **Coverage:** 
  - Simple views block parsing
  - Include/exclude expressions
  - Styles block parsing
  - Optional views (architecture without views block)

## Examples

### Example Files

1. **`examples/implied_relationships.sruja`**
   - Demonstrates implied relationships
   - Shows how parent relationships are automatically inferred

2. **`examples/views_customization.sruja`**
   - Demonstrates optional views block
   - Shows filtering, styling, and autolayout

3. **`examples/README_VIEWS.md`**
   - Documentation for using the new features
   - Usage examples and best practices

## Usage

### Implied Relationships

No special syntax needed - it works automatically:

```sruja
Customer -> Shop.WebApp "Uses"
// Automatically infers: Customer -> Shop
```

### Views Block

Views are optional - only define if you need customization:

```sruja
// Without views block - standard C4 views are automatic
architecture "Simple" {
    system API "API Service" { ... }
}

// With views block - for customization
architecture "Custom" {
    system API "API Service" { ... }
    
    views {
        container API "Custom View" {
            include *
        }
    }
}
```

## Export Integration

### SVG Export

- Views block provides styling hints
- View expressions can be used for filtering (future enhancement)
- Styles are applied by tag

### Markdown Export

- Checks for custom views before generating standard views
- Uses custom views if defined
- Falls back to automatic C4 views if no views block

### JSON Export

- Views block is included in extended mode
- View definitions are serialized

## Design Philosophy Alignment

### Implied Relationships

- ✅ **DRY Principle** - Reduces repetition
- ✅ **Progressive Disclosure** - Works automatically, no configuration needed
- ✅ **Simplicity** - Reduces boilerplate while maintaining clarity

### Views Block

- ✅ **Start Simple** - Views are optional, C4 views are automatic
- ✅ **Progressive Disclosure** - Advanced users can customize when needed
- ✅ **Approachability** - Beginners don't need to think about views

## Future Enhancements

1. **Full View Expression Evaluation**
   - Pattern matching (e.g., `"->Element->"`)
   - Type-based filtering (e.g., `element.type==container`)
   - Parent-based filtering (e.g., `element.parent==System`)

2. **SVG Export Integration**
   - Apply view expressions for filtering elements
   - Apply styles from views block to SVG nodes

3. **View Export by Name**
   - Export specific views by name from CLI
   - Support for multiple named views

4. **View Validation**
   - Validate view expressions against model
   - Warn about invalid element references

## Testing

All features are tested:

- ✅ Implied relationships inference
- ✅ Views block parsing
- ✅ View expressions (include/exclude)
- ✅ Styles block parsing
- ✅ Optional views (backward compatibility)

Run tests:
```bash
go test ./pkg/language -v
go test ./pkg/export/views -v
```

## Summary

Both features are implemented and tested:

1. **Implied Relationships** - ✅ Complete
   - Automatically infers parent relationships
   - Reduces boilerplate
   - Follows DRY principle

2. **Optional Views Block** - ✅ Complete
   - Optional customization
   - C4 views remain automatic
   - Integrated with markdown export
   - Helper functions for SVG export

The implementation maintains backward compatibility - existing DSL files continue to work without changes.

# Views and Implied Relationships Examples

This directory contains examples demonstrating the new features:

## Implied Relationships

**File:** `demo_implied_relationships.sruja`

Demonstrates how implied relationships reduce boilerplate:

```sruja
// Only need to define:
Customer -> Shop.WebApp "Uses"
Customer -> Shop.API "Calls"

// Automatically inferred:
//   Customer -> Shop (implied from Customer -> Shop.WebApp)
//   Customer -> Shop (implied from Customer -> Shop.API)
```

**Benefits:**
- Reduces repetition
- Follows DRY principle
- Maintains clarity

## Views Block (Optional Customization)

**File:** `demo_views_customization.sruja`

Demonstrates optional views block for customizing diagrams:

```sruja
views {
    // Custom filtered view
    container Shop "API Focus" {
        include Shop.API Shop.DB Shop.Cache
        exclude Shop.WebApp Shop.PaymentService
        autolayout lr
    }
    
    // Custom styling
    styles {
        element "Database" {
            shape "cylinder"
            color "#ff0000"
        }
    }
}
```

**Key Points:**
- Views are **OPTIONAL** - if omitted, standard C4 views are automatically generated
- Only needed for customization (filtering, styling, etc.)
- Aligns with Sruja's "start simple" philosophy

## Usage

### Preview Views in the Website

```bash
# Use the website preview to render standard views automatically
# and apply custom views/styling when present
```

### Export Extended JSON

When a views block is defined in the DSL, the JSON export can include view definitions:

```bash
sruja export json --extended examples/demo_views_customization.sruja
```

### View Types

- `systemContext` - System context view (C4 L1)
- `container` - Container view (C4 L2)
- `component` - Component view (C4 L3)
- `deployment` - Deployment view

### View Expressions

- `include *` - Include all elements in scope
- `include Element1 Element2` - Include specific elements
- `exclude Element1` - Exclude specific elements
- `autolayout lr|tb|auto` - Layout direction hint

### Styling

Styles are applied by tag:

```sruja
styles {
    element "Database" {
        shape "cylinder"
        color "#ff0000"
        stroke "#cc0000"
        strokeWidth "2"
    }
}
```

## Integration with Exports

The views block is integrated with:

1. **Website Preview** - Renders diagrams and applies styles/filters in the browser
2. **JSON Export** - Includes view definitions in extended mode (`--extended`)

## Best Practices

1. **Start Simple** - Don't define views unless you need customization
2. **Use Tags** - Tag elements to apply styles consistently
3. **Named Views** - Use descriptive names for views
4. **Progressive Disclosure** - Add views as your architecture grows

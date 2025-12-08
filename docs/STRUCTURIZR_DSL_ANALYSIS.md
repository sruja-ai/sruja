# Structurizr DSL Analysis & Enhancement Opportunities

## Executive Summary

This document analyzes the [Structurizr DSL tutorial](https://docs.structurizr.com/dsl/tutorial) and identifies concepts that could enhance the Sruja DSL. The analysis focuses on practical improvements that align with Sruja's design philosophy of simplicity and progressive disclosure.

### Key Finding: C4 Views Should Be Automatic

**Critical Design Decision:** Unlike Structurizr (where users must explicitly define all views), **Sruja should automatically generate C4 standard views** from the model structure. This aligns perfectly with Sruja's "start simple" philosophy:

- ✅ **Beginners don't need to think about views** - they're automatically generated
- ✅ **C4 standard views are well-defined** - L1 (System Context), L2 (Container), L3 (Component) can be inferred
- ✅ **Progressive disclosure** - Advanced users can customize views when needed, but it's optional

**Current Sruja behavior is correct:** Views are auto-generated via CLI flags (`-view=c4:l1`, `-view=c4:l2:API`). This should remain the default.

**Enhancement:** Add an **optional** `views` block **only for customization** (filtering, styling, non-standard views). If omitted, standard C4 views are automatically available.

## Key Concepts from Structurizr DSL

### 1. Workspace/Model/Views Separation

**Structurizr Approach:**
```structurizr
workspace "Name" "Description" {
    model {
        // Elements and relationships
    }
    
    views {
        // View definitions (REQUIRED - must define all views)
        systemContext ss "Diagram1" { ... }
        container ss "Diagram2" { ... }
    }
}
```

**Current Sruja Approach:**
```sruja
architecture "Name" {
    // Elements and relationships
    // Views are automatically generated from model structure
}

// Views accessed via CLI:
// sruja export -view=c4:l1 svg file.sruja  // Auto-generated system context
// sruja export -view=c4:l2:API svg file.sruja  // Auto-generated container view
```

**Analysis:**
- ✅ **Sruja's current approach is actually BETTER for beginners** - views are automatically inferred from the model
- C4 standard views (L1: System Context, L2: Container, L3: Component) are well-defined and can be automatically generated
- Users don't need to think about views unless they want customization
- This aligns with Sruja's "start simple" philosophy - a 1st-year CS student shouldn't need to define views

**Enhancement Opportunity:** ⭐⭐ (Medium Value - Optional Customization Only)
- **Keep automatic C4 views as default** (current behavior is good!)
- Add **optional** `views` block **only for customization**:
  - Custom filtering (include/exclude specific elements)
  - Custom styling
  - Non-standard views (e.g., filtered views, custom groupings)
- Views block should be **optional** - if not present, generate standard C4 views automatically

### 2. Implied Relationships (DRY Principle)

**Structurizr Approach:**
```structurizr
// Only need to define:
u -> ss.wa "Uses"
ss.wa -> ss.db "Reads/Writes"

// Implied: u -> ss (because wa is inside ss)
```

**Current Sruja Approach:**
```sruja
User -> API "Uses"
User -> API.WebApp "Uses"  // Must be explicit
API.WebApp -> API.Database "Reads/Writes"
```

**Analysis:**
- Sruja requires all relationships to be explicit
- This can lead to repetition (e.g., `User -> System` and `User -> System.Container`)
- Structurizr automatically infers parent relationships when child relationships exist

**Enhancement Opportunity:** ⭐⭐⭐ (High Value)
- Implement implied relationship inference
- Reduces boilerplate while maintaining clarity
- Aligns with DRY principle

### 3. Hierarchical Identifiers

**Structurizr Approach:**
```structurizr
!identifiers hierarchical

system ss "Software System" {
    container wa "Web Application"
    container db "Database"
}

// Can reference as: ss.wa, ss.db
```

**Current Sruja Approach:**
```sruja
system API "API Service" {
    container WebApp "Web Application"
    container Database "Database"
}

// Already supports: API.WebApp, API.Database
```

**Analysis:**
- ✅ Sruja already supports hierarchical identifiers via dot notation
- No enhancement needed - this is already implemented

### 4. View Expressions (include/exclude)

**Structurizr Approach:**
```structurizr
views {
    container ss "Diagram2" {
        include *  // Default set
        // OR
        include u ss.wa ss.db  // Explicit elements
        // OR
        include "->ss.wa->"  // Element + dependencies
        // OR
        include "->element.type==container->"  // By type
        // OR
        include "->element.parent==ss->"  // By parent
    }
}
```

**Current Sruja Approach:**
```sruja
// Views are implicit based on export command:
// sruja export -view=c4:l1 svg file.sruja
// sruja export -view=container:API svg file.sruja
```

**Analysis:**
- Sruja views are command-line driven, not DSL-driven
- No fine-grained control over what elements appear in views
- No ability to create custom views with specific element filters

**Enhancement Opportunity:** ⭐⭐⭐ (High Value)
- Add `views` block with `include`/`exclude` expressions
- Support patterns like:
  - `include *` (default set)
  - `include Element1 Element2` (explicit)
  - `include "->Element->"` (element + dependencies)
  - `include "->element.type==container->"` (by type)
  - `exclude Element1` (exclusions)

### 5. Element Styling with Tags

**Structurizr Approach:**
```structurizr
// Add tag to element
container db "Database Schema" {
    tags "Database"
}

// Style by tag
views {
    styles {
        element "Database" {
            shape cylinder
            color #ff0000
        }
    }
}
```

**Current Sruja Approach:**
```sruja
container Database "PostgreSQL Database" {
    tags ["Database", "PostgreSQL"]
    // Styling not yet defined in DSL
}
```

**Analysis:**
- Sruja supports tags on elements
- No explicit styling system in DSL (styling may be handled in viewer/export)
- Tags exist but may not be used for styling

**Enhancement Opportunity:** ⭐⭐ (Medium Value)
- Add `styles` block to `views` section
- Allow styling elements by tag
- Support color, shape, stroke, etc.

### 6. View Types

**Structurizr Approach:**
```structurizr
views {
    systemContext ss "Diagram1" { ... }
    container ss "Diagram2" { ... }
    component ss.wa "Diagram3" { ... }
    dynamic ss "Diagram4" { ... }
    deployment ss "Diagram5" { ... }
}
```

**Current Sruja Approach:**
```sruja
// Views specified via CLI:
// -view=c4:l1 (system context)
// -view=c4:l2:API (container)
// -view=c4:l3:API/WebApp (component)
// -view=deployment:Prod (deployment)
```

**Analysis:**
- Sruja supports all these view types via CLI
- No DSL-level view definitions
- No named views that can be referenced

**Enhancement Opportunity:** ⭐⭐⭐ (High Value)
- Add view type keywords: `systemContext`, `container`, `component`, `deployment`
- Allow named views: `container API "Container View" { ... }`
- Views become first-class DSL constructs

## Recommended Enhancements (Prioritized)

### Priority 1: High Value, High Impact

#### 1.1 Implied Relationships ⭐⭐⭐
**Why:** Reduces boilerplate, follows DRY principle, aligns with Structurizr.

**Proposed Behavior:**
```sruja
// Explicit relationships:
Customer -> Shop.WebApp "Uses"
Shop.WebApp -> Shop.DB "Reads/Writes"

// Automatically inferred:
// Customer -> Shop (implied, because WebApp is inside Shop)
```

**Implementation Notes:**
- Make this optional via a flag or directive: `!implied-relationships`
- Only infer parent relationships when child relationships exist
- Maintain explicit relationships for clarity when needed

**Benefits:**
- Reduces repetition
- Follows DRY principle
- Still allows explicit relationships when needed for clarity

#### 1.2 Optional Views Block (For Customization Only) ⭐⭐
**Why:** Allows customization while keeping simplicity for beginners.

**Key Design Decision:** 
- **C4 standard views should be AUTOMATIC by default** (current behavior is correct!)
- Views block should be **OPTIONAL** - only needed for customization
- This aligns with Sruja's "start simple" philosophy

**Proposed Syntax (Optional - only for customization):**
```sruja
architecture "E-commerce Platform" {
    person Customer "Customer"
    system Shop "E-commerce Shop" {
        container WebApp "Web Application"
        container API "API Gateway"
        datastore DB "PostgreSQL Database"
    }
    
    Customer -> Shop.WebApp "Uses"
    Shop.WebApp -> Shop.API "Calls"
    Shop.API -> Shop.DB "Reads/Writes"
    
    // OPTIONAL: Only define views if you need customization
    // If omitted, standard C4 views are auto-generated:
    // - System Context (L1) for each system
    // - Container (L2) for each system with containers
    // - Component (L3) for each container with components
    
    views {
        // Custom filtered view (only show specific elements)
        container Shop "API Focus" {
            include Shop.API Shop.DB
            exclude Shop.WebApp
        }
        
        // Custom styling
        styles {
            element "Database" {
                shape cylinder
                color "#ff0000"
            }
        }
    }
}
```

**Benefits:**
- Beginners don't need to think about views (automatic C4 views)
- Advanced users can customize when needed
- Progressive disclosure - simple by default, powerful when needed
- Views are version-controlled when explicitly defined


### Priority 2: Medium Value, Good Impact

#### 2.1 View Expressions (include/exclude) - Only in Custom Views
**Why:** Provides fine-grained control when customizing views.

**Note:** These are only needed when defining custom views. Standard C4 views use automatic `include *` behavior.

**Proposed Syntax (only in optional views block):**
```sruja
views {
    // Custom view with filtering
    container Shop "API Focus" {
        include Shop.API Shop.DB
        include "->Shop.DB->"  // DB + dependencies
        exclude Shop.WebApp  // Remove if needed
    }
    
    // Custom view by type
    container Shop "Containers Only" {
        include "->element.type==container->"
    }
    
    // Custom view - children only
    container Shop "Internal Only" {
        include "->element.parent==Shop->"
    }
}
```

**Benefits:**
- Flexible view composition for advanced use cases
- Can create focused views for specific audiences
- Supports complex filtering logic
- **Only needed when customizing** - standard views are automatic

#### 2.2 Element Styling
**Why:** Allows visual customization directly in DSL.

**Proposed Syntax:**
```sruja
views {
    styles {
        element "Database" {
            shape cylinder
            color "#ff0000"
            stroke "#cc0000"
            strokeWidth 2
        }
        
        element "Container" {
            shape roundedbox
            color "#0773af"
        }
        
        relationship "Relationship" {
            thickness 3
            color "#666666"
        }
    }
}
```

**Benefits:**
- Visual customization in DSL
- Consistent styling across exports
- Tag-based styling (reusable)

### Priority 3: Lower Priority

#### 3.1 View Autolayout
**Why:** Provides layout hints for diagram generation.

**Proposed Syntax:**
```sruja
views {
    container Shop "Container View" {
        include *
        autolayout lr  // left-to-right
        // OR
        autolayout tb  // top-to-bottom
        // OR
        autolayout auto  // automatic
    }
}
```

**Note:** This may already be handled by the diagram renderer. Check if this adds value.

## Implementation Considerations

### Backward Compatibility
- All enhancements should be optional
- Existing DSL files should continue to work
- CLI view flags should still work (for compatibility)

### Syntax Design
- Follow Sruja's philosophy: simple, intuitive, progressive disclosure
- Views block should be optional (don't break existing files)
- Use familiar keywords (`include`, `exclude`, `autolayout`)

### Parser Changes
- Add `views` block to AST
- Add view expression parsing
- Add style block parsing
- Maintain backward compatibility

### Export Changes
- Support named views from DSL
- Fall back to CLI flags if no views defined
- Apply styles from DSL to exports

## Comparison Table

| Feature | Structurizr | Sruja (Current) | Enhancement Needed? |
|---------|-------------|-----------------|---------------------|
| Workspace/Model/Views separation | ✅ (required) | ✅ (automatic views) | ✅ **Better approach - keep automatic** |
| Implied relationships | ✅ | ❌ | ⭐⭐⭐ **Yes - High Priority** |
| Hierarchical identifiers | ✅ | ✅ | ✅ Already supported |
| View expressions (include/exclude) | ✅ (required) | ⚠️ (automatic only) | ⭐⭐ **Optional - for customization** |
| Element styling | ✅ | ⚠️ (partial) | ⭐⭐ Yes - for customization |
| Named views | ✅ (required) | ⚠️ (auto-generated) | ⭐ **Optional - only for custom views** |
| C4 standard views | ✅ (must define) | ✅ (automatic) | ✅ **Sruja's approach is better!** |
| Autolayout hints | ✅ | ⚠️ (renderer handles) | ⭐ Optional |

## Conclusion

The Structurizr DSL tutorial reveals several valuable concepts that could enhance Sruja:

### Key Insight: C4 Views Should Be Automatic

**Important Design Decision:** Unlike Structurizr (where views must be explicitly defined), Sruja should **automatically generate C4 standard views** from the model. This aligns with Sruja's "start simple" philosophy:

- ✅ **Beginners:** Don't need to think about views - they're automatic
- ✅ **C4 Standard:** L1 (System Context), L2 (Container), L3 (Component) are well-defined and can be inferred
- ✅ **Progressive Disclosure:** Advanced users can customize views when needed, but it's optional

### Recommended Enhancements:

1. **High Priority:** 
   - ✅ **Implied relationships** - Reduces boilerplate (DRY principle)
   - ⭐ **Optional views block** - Only for customization, not required

2. **Medium Priority:** 
   - View expressions (include/exclude) - Only in custom views
   - Element styling - For visual customization

3. **Low Priority:** 
   - Autolayout hints (may already be handled by renderer)

### Design Philosophy Alignment

This approach perfectly aligns with Sruja's core principles:
- **"Start simple, stay simple"** - Views are automatic, no configuration needed
- **"Progressive disclosure"** - Customize only when you need to
- **"Approachability first"** - A 1st-year CS student doesn't need to define views

The key is: **Make C4 views automatic, make customization optional.**

## Next Steps

1. **Design Phase:**
   - Create detailed syntax specification for `views` block
   - Design implied relationship inference algorithm
   - Plan backward compatibility strategy

2. **Implementation Phase:**
   - Add `views` block to parser
   - Implement view expression evaluation
   - Add implied relationship inference
   - Update export commands to use DSL views

3. **Testing Phase:**
   - Test with existing DSL files (backward compatibility)
   - Test new view features
   - Update examples and documentation

4. **Documentation:**
   - Update LANGUAGE_SPECIFICATION.md
   - Add examples showing new features
   - Create migration guide (if needed)

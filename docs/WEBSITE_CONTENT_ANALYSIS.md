# Website Content Analysis: Clarity, Accuracy & Consistency

## Executive Summary

The website is **generally good** but has some areas for improvement to make it more beginner-friendly and ensure consistency with the DSL.

## Strengths ✅

1. **Clear Value Proposition**: Homepage clearly states "Architecture-as-Code"
2. **Good Structure**: Getting Started → Tutorials → Courses progression
3. **Beginner Path**: Dedicated beginner path document exists
4. **Examples**: Real-world examples available
5. **Multiple Entry Points**: Designer, CLI, VS Code extension

## Issues Found

### 1. Syntax Inconsistency ⚠️

**Problem**: Documentation shows two different syntax approaches inconsistently:

- **Approach 1**: `kind` declarations (shown in getting-started.md, cheatsheet)

  ```sruja
  person = kind "Person"
  system = kind "System"
  ```

- **Approach 2**: `stdlib` imports (shown in dsl-basics.md, intro.md)
  ```sruja
  import { * } from 'sruja.ai/stdlib'
  ```

**Impact**: Beginners may be confused about which approach to use.

**Recommendation**:

- Standardize on `stdlib` imports as the recommended approach (cleaner, less verbose)
- Show `kind` declarations as an alternative for advanced users
- Update all examples to use `stdlib` imports first

### 2. Homepage Could Be More Beginner-Friendly ⚠️

**Current Issues**:

- Uses technical terms: "Architecture-as-Code", "bidirectional sync"
- Doesn't show a simple example immediately
- Value props are good but could be more concrete

**Recommendation**:

- Add a simple visual example on homepage
- Show "What you write" vs "What you get" side-by-side
- Add a "See it in action" button that opens Designer with a simple example

### 3. Getting Started Could Be Clearer ⚠️

**Current Issues**:

- Uses `kind` declarations (more verbose)
- Doesn't mention `stdlib` imports as the easier option
- Example is good but could be simpler

**Recommendation**:

- Start with `stdlib` imports
- Show the simplest possible example first
- Add a "Try in Designer" link

### 4. Missing Quick Visual Demo ⚠️

**Problem**: No immediate visual demonstration on homepage

**Recommendation**:

- Add interactive diagram on homepage
- Show live example that updates as you type
- "Try it now" CTA that opens Designer

### 5. Terminology Consistency ⚠️

**Issues**:

- Some docs use "Datastore", others use "Database"
- Inconsistent use of "Container" vs "container"
- "ADRs" not explained on first mention

**Recommendation**:

- Standardize terminology across all docs
- Add glossary or tooltips for technical terms
- Explain acronyms on first use

## Specific Recommendations

### Homepage Improvements

1. **Add Visual Example**:

   ```sruja
   import { * } from 'sruja.ai/stdlib'

   User = person "Customer"
   App = system "E-commerce Platform" {
     Web = container "React App"
     API = container "Node.js API"
     DB = database "PostgreSQL"
   }

   User -> App.Web "Visits"
   App.Web -> App.API "Requests"
   App.API -> App.DB "Stores Data"
   ```

   Show this code → diagram transformation

2. **Simplify Value Props**:
   - "Write code, get diagrams" (instead of "bidirectional sync")
   - "Never outdated" (instead of "prevent drift")
   - "Version controlled" (clearer than "single source of truth")

3. **Add "Try Now" Section**:
   - Interactive code editor on homepage
   - Live diagram preview
   - "Open in Designer" button

### Getting Started Improvements

1. **Use stdlib imports**:

   ```sruja
   import { * } from 'sruja.ai/stdlib'
   ```

   Instead of `kind` declarations

2. **Simpler first example**:
   - Start with just person → system
   - Then add containers
   - Progressive complexity

3. **Add visual feedback**:
   - Show what the diagram looks like
   - Link to Designer with pre-filled example

### Documentation Consistency

1. **Standardize Syntax**:
   - All examples should use `stdlib` imports
   - Show `kind` declarations as alternative in advanced section

2. **Terminology**:
   - Use "database" consistently (not "datastore" unless specifically needed)
   - Explain "ADRs" on first mention
   - Use consistent capitalization

3. **Examples**:
   - All examples should be runnable
   - Test all examples against current DSL
   - Update examples if DSL changes

## Accuracy Check

### DSL Syntax Accuracy ✅

- Element declarations: ✅ Correct
- Relations: ✅ Correct
- Nested elements: ✅ Correct
- Views: ✅ Correct
- Metadata: ✅ Correct

### Inconsistencies Found

1. **Kind Declarations vs Imports**: Mixed usage
2. **Database vs Datastore**: Inconsistent
3. **Example Complexity**: Some examples too complex for beginners

## Beginner-Friendliness Score

**Current**: 7/10
**Target**: 9/10

### What's Good

- Clear value proposition
- Good structure
- Beginner path exists
- Examples available

### What Needs Improvement

- Syntax inconsistency
- No immediate visual demo
- Technical jargon on homepage
- Getting started could be simpler

## Action Items

### High Priority

1. ✅ Standardize on `stdlib` imports in all examples
2. ✅ Add visual example to homepage
3. ✅ Simplify getting started guide
4. ✅ Add "Try Now" interactive demo

### Medium Priority

1. ✅ Update terminology for consistency
2. ✅ Add tooltips/explanations for technical terms
3. ✅ Test all examples against current DSL
4. ✅ Add glossary

### Low Priority

1. ✅ Add video walkthrough
2. ✅ Add "What you'll build" preview
3. ✅ Add success stories/testimonials

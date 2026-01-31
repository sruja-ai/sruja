# Code Review Checklist

**Purpose:** Prevent issues identified in code robustness research  
**Based On:** [Code Robustness Research Report](./CODE_ROBUSTNESS_RESEARCH.md)  
**Last Updated:** January 2025  
**Status:** Active - Use for all code reviews

---

## Quick Reference Legend

- 🔴 **CRITICAL** - Must pass before merge approval
- 🟡 **HIGH** - Should pass unless explicitly waived
- 🟢 **MEDIUM** - Recommended but can be deferred
- ⚪ **LOW** - Nice to have improvements

---

## Part 1: Rust Code Review Checklist

### Error Handling 🔴

- [ ] **No `.unwrap()` in production code**
  - Exception: Tests may use `.unwrap()` if explicitly documenting test assumptions
  - Use `?` operator for Result propagation
  - Use `.unwrap_or_else()`, `.unwrap_or_default()` only with documented fallbacks
  
- [ ] **No `.expect()` except for programming errors**
  - Document why the expectation is invariant
  - Use `thiserror` for custom error types
  - Add `# Errors` section to function documentation

- [ ] **All fallible operations return `Result<T, E>`**
  - File I/O operations
  - Parsing operations
  - External API calls
  - Serialization/deserialization

### Type Safety 🔴

- [ ] **No raw `panic!` calls in application code**
  - Use `Result` instead
  - Document panic reasons if in library code with `# Panics`

- [ ] **Borrowing rules followed**
  - Prefer `&T` over `&String`, `&[T]` over `&Vec<T>`
  - Use `Cow<'a, T>` for conditional ownership
  - Avoid excessive clones - document if necessary

- [ ] **Lifetime elision preferred**
  - Use elision where compiler infers correctly
  - Only add explicit lifetimes when necessary

### API Design 🟡

- [ ] **Public functions documented**
  - `///` doc comments for all public items
  - `# Examples` section with runnable code
  - `# Errors` section for fallible functions
  - `# Panics` section if applicable

- [ ] **Traits properly designed**
  - Sealed traits where appropriate (`#[non_exhaustive]`)
  - Extension traits for adding methods to foreign types
  - Implement `From` instead of `Into` (auto-derived)

- [ ] **Error types are comprehensive**
  - Use `thiserror` for library errors
  - Use `anyhow` for application errors
  - Chain underlying errors with `#[source]`
  - Add context with `.context()` or `.with_context()`

### Memory & Performance 🟡

- [ ] **Allocation-aware**
  - Use `with_capacity()` when size is known
  - Prefer `SmallVec` for usually-small collections
  - Use `clone_from()` to reuse allocations
  - Reuse collections with `clear()` in loops

- [ ] **String operations optimized**
  - Avoid `format!()` when string literals work
  - Use `write!()` instead of `format!()` in hot paths
  - Use `CompactString` for small string optimization

- [ ] **Type sizes considered**
  - Box large enum variants to reduce type size
  - Use `Box<[T]>` instead of `Vec<T>` when fixed
  - Consider `ThinVec` for often-empty vectors

### Testing 🟢

- [ ] **Tests for critical paths**
  - Unit tests for all public functions
  - Integration tests for complex workflows
  - Property-based tests for data transformations

- [ ] **Test quality**
  - Descriptive test names
  - Arrange/Act/Assert structure
  - Tests don't use `unwrap()` unless documenting assumptions
  - Edge cases covered

- [ ] **Async tests use `tokio::test`**
  - All async code tested with proper runtime
  - Timeouts for long-running tests

### Specific Issue Prevention 🔴

- [ ] **No compilation errors**
  - All code compiles without errors
  - Warnings are addressed or explicitly suppressed

- [ ] **Parser position tracking implemented**
  - `SourceLocation` has real line/column numbers
  - TODOs for position tracking are resolved
  - Error messages include accurate locations

- [ ] **TODO/FIXME comments addressed**
  - No TODO/FIXME in production code
  - If kept, must have tracking ticket number
  - Document why not immediately solvable

- [ ] **Complete feature implementations**
  - No stub functions returning default values
  - All documented features are functional
  - Placeholder text removed from user-facing strings

---

## Part 2: React/TypeScript Code Review Checklist

### Type Safety 🔴

- [ ] **No `any` types**
  - Use proper TypeScript types
  - Create newtypes if needed
  - Use `unknown` instead of `any` when type is truly unknown

- [ ] **No type assertions without checks**
  - Prefer type guards: `if (typeof x === 'string')`
  - Use type predicates when necessary
  - Document why assertion is safe if unavoidable

- [ ] **No `@ts-ignore` or `@ts-expect-error`**
  - Exception: Temporary during refactoring with tracking ticket
  - Must include comment explaining why suppression needed
  - Should be removed ASAP

- [ ] **Proper interface/type definitions**
  - All props typed with interfaces
  - No implicit `any` in function parameters
  - Strict null checks enabled

### Error Handling 🔴

- [ ] **All async operations error handled**
  - No empty catch blocks
  - User-friendly error messages displayed
  - Errors logged to proper monitoring service
  - Graceful degradation for non-critical failures

- [ ] **Error boundaries for major sections**
  - ErrorBoundary wraps major UI components
  - Fallback UIs for error states
  - Errors reported to monitoring

- [ ] **API failures handled**
  - Network errors caught and displayed
  - Timeout handling
  - Retry logic for transient failures
  - Loading states for async operations

### Component Design 🟡

- [ ] **Component complexity manageable**
  - Components under 300 lines
  - Single responsibility principle
  - Extract reusable pieces
  - Proper use of hooks for logic separation

- [ ] **Props validation**
  - PropTypes not needed if TypeScript used
  - Required props clearly marked
  - Default values provided where appropriate
  - No prop drilling for deeply nested values

- [ ] **State management appropriate**
  - Local state for component-local data
  - Global state (Zustand) for app-wide state
  - Server state (React Query) for API data
  - No unnecessary re-renders

### Performance 🟡

- [ ] **Unnecessary re-renders avoided**
  - Use `React.memo` for expensive components
  - `useCallback` for event handlers passed down
  - `useMemo` for expensive computations
  - Keys on list items are stable

- [ ] **Bundle size considered**
  - Tree-shaking enabled
  - Lazy loading for large components
  - No unnecessary dependencies
  - Code splitting for routes

- [ ] **No console.* in production**
  - Remove all `console.log`, `console.warn`, `console.error`
  - Use proper logger service
  - Add production-only log filtering

### Specific Issue Prevention 🔴

- [ ] **No incomplete features**
  - TODO/FIXME addressed or tracked
  - Stub implementations removed
  - Placeholder text removed
  - Unconnected handlers implemented or UI removed

- [ ] **Critical functionality complete**
  - Delete operations functional
  - Edit operations functional
  - CRUD operations for all major entities
  - All buttons/interactions have handlers

- [ ] **User feedback provided**
  - Loading states for async operations
  - Success/error toasts or notifications
  - Confirmation dialogs for destructive actions
  - No silent failures

- [ ] **Accessibility considered**
  - Keyboard navigation
  - ARIA labels where needed
  - Focus management
  - Screen reader friendly

### Testing 🟢

- [ ] **Component tests**
  - Unit tests for pure functions
  - Component tests with React Testing Library
  - User interactions tested
  - Error states tested

- [ ] **E2E tests for critical paths**
  - Playwright tests for major workflows
  - Tests cover happy paths
  - Tests cover error cases

---

## Part 3: Cross-Cutting Concerns

### Documentation 🟡

- [ ] **Changes documented**
  - README updated if API changes
  - Changelog entry for user-facing changes
  - Inline code comments for complex logic

- [ ] **APIs documented**
  - Public functions have JSDoc
  - Props have descriptions
  - Complex hooks have usage examples

### Security 🔴

- [ ] **No sensitive data in logs**
  - No passwords, tokens, or secrets logged
  - User data handled appropriately
  - Input validation on all user inputs

- [ ] **Dependencies vetted**
  - No known vulnerabilities (check with `npm audit` / `cargo audit`)
  - Dependencies actively maintained
  - Minimal dependency surface area

### Code Quality 🟢

- [ ] **Code style consistent**
  - `cargo fmt` / `prettier` run
  - No linting warnings
  - Naming follows conventions

- [ ] **Dead code removed**
  - No commented-out code
  - No unused imports
  - No unused functions/variables

---

## Part 4: Merge Approval Criteria

### Must-Have Before Merge 🔴

- [ ] **No compilation errors**
- [ ] **All tests passing**
- [ ] **No `unwrap()` in production Rust code**
- [ ] **No `any` types in TypeScript**
- [ ] **No console.* in production React code**
- [ ] **Error handling for all fallible operations**
- [ ] **No empty catch blocks**
- [ ] **Critical features functional**
- [ ] **No TODO/FIXME without tracking ticket**

### Should-Have Before Merge 🟡

- [ ] **Tests for new functionality**
- [ ] **Documentation updated**
- [ ] **Performance reviewed**
- [ ] **Accessibility considered**
- [ ] **Code coverage not decreased**

---

## Part 5: Quick Reference - Anti-Patterns to Reject

### Rust Anti-Patterns 🚫

```rust
// ❌ DON'T - unwrap in production
let value = some_option.unwrap();

// ✅ DO - proper error handling
let value = some_option.ok_or_else(|| Error::MissingValue)?;

// ❌ DON'T - expect on expected errors
let result = parse(input).expect("Should parse");

// ✅ DO - return Result
let result = parse(input)?;

// ❌ DON'T - panic on errors
if value < 0 {
    panic!("Value cannot be negative");
}

// ✅ DO - return error
if value < 0 {
    return Err(Error::InvalidValue(value));
}

// ❌ DON'T - clone unnecessarily
let title = elem.title.clone().unwrap_or_else(|| elem.name.clone());

// ✅ DO - borrow when possible
let title = elem.title.as_deref().unwrap_or(&elem.name);
```

### React/TypeScript Anti-Patterns 🚫

```typescript
// ❌ DON'T - any types
const processData = (data: any) => {
  // ...
};

// ✅ DO - proper types
interface Data {
  id: string;
  value: number;
}
const processData = (data: Data) => {
  // ...
};

// ❌ DON'T - empty catch blocks
try {
  await api.call();
} catch (error) {
  // silent failure
}

// ✅ DO - proper error handling
try {
  await api.call();
} catch (error) {
  logger.error("API call failed", error);
  showToast("Operation failed", "error");
}

// ❌ DON'T - console.log in production
console.log("Debug info", data);

// ✅ DO - proper logging
logger.info("Processing data", { id: data.id });

// ❌ DON'T - ignore errors silently
await operation(); // no error handling

// ✅ DO - handle errors
try {
  await operation();
} catch (error) {
  // handle error appropriately
}
```

---

## Part 6: Common Issues Found in Codebase

### Issues to Watch For (Based on Research)

**Rust:**
- [ ] SourceLocation tracking incomplete (parser.rs)
- [ ] Missing position tracking causing poor error messages
- [ ] Incomplete TODO implementations in exporter
- [ ] Change management using placeholder text
- [ ] Environment variable fallbacks too permissive

**React:**
- [ ] Delete functionality stubbed (UnifiedDetailsList)
- [ ] Edit handlers unconnected (NavigationPanel)
- [ ] Metadata deletion not implemented (OverviewPanel)
- [ ] FQN resolution incomplete (fqnResolver.ts)
- [ ] Clipboard errors silent (DslPreview.tsx)
- [ ] Score calculation errors silent (multiple governance components)

---

## Part 7: Reviewer Responsibilities

### Reviewer Checklist
- [ ] Verify all items in this checklist are addressed
- [ ] Leave specific feedback on any violations
- [ ] Suggest improvements beyond checklist items
- [ ] Verify tests cover new functionality
- [ ] Check documentation is updated
- [ ] Confirm no regression in existing functionality

### Approver Responsibility
- [ ] Only approve if all 🔴 items pass
- [ ] Flag 🟡 items that should be addressed
- [ ] Note 🟢 items for future improvement
- [ ] Comment on overall code quality
- [ ] Verify PR description accurately describes changes

---

## Part 8: Post-Merge Actions

### After Merge 📋

- [ ] Monitor error rates in production
- [ ] Check performance metrics
- [ ] Update documentation if needed
- [ ] Close related tickets
- [ ] Schedule follow-up if any items deferred

---

## Appendix: Resources

- [Rust Skills Guidelines](../AGENTS.md) - Comprehensive Rust best practices
- [Code Robustness Research](./CODE_ROBUSTNESS_RESEARCH.md) - Detailed analysis
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/intro.html)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)

---

**Checklist Version:** 1.0  
**Last Updated:** January 2025  
**Review Cycle:** Quarterly or after major framework updates  
**Maintainer:** Tech Lead
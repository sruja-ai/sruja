# Input Validation and Sanitization Implementation

## Overview

Comprehensive input validation and sanitization has been implemented to ensure FAANG-level security and data integrity.

## Implementation

### Validation Utility Module

Created `inputValidation.ts` with comprehensive validation functions:

#### 1. Node ID Validation
- **Function**: `validateNodeId(id: string)`
- **Rules**:
  - Must be alphanumeric, underscore, or hyphen only
  - Length: 1-100 characters
  - Required (cannot be empty)
- **Use Case**: Validates node IDs when creating/renaming nodes

#### 2. Node Label Validation
- **Function**: `validateNodeLabel(label: string)`
- **Rules**:
  - Removes HTML tags (XSS prevention)
  - Length: 1-200 characters
  - Required (cannot be empty)
- **Use Case**: Validates labels for all node types

#### 3. Relation Label Validation
- **Function**: `validateRelationLabel(label: string)`
- **Rules**:
  - Sanitizes HTML
  - Length: 0-200 characters (optional)
- **Use Case**: Validates relation labels

#### 4. DSL Input Validation
- **Function**: `validateDslInput(dsl: string)`
- **Rules**:
  - Maximum length: 10MB
  - Basic structure checks
- **Use Case**: Validates DSL before parsing

#### 5. Search Query Validation
- **Function**: `validateSearchQuery(query: string)`
- **Rules**:
  - Sanitizes HTML
  - Maximum length: 200 characters
  - Empty query is valid
- **Use Case**: Validates search input

#### 6. URL Validation
- **Function**: `validateUrl(url: string)`
- **Rules**:
  - Must be valid URL format
  - Only http/https protocols allowed
- **Use Case**: Validates share links

#### 7. ADR Data Validation
- **Function**: `validateAdrData(data: AdrValidationData)`
- **Rules**:
  - Validates title (required, sanitized)
  - Validates status (must be: proposed, accepted, deprecated, superseded)
  - Sanitizes all text fields (context, decision, consequences)
- **Use Case**: Validates ADR creation/editing

#### 8. Properties Update Validation
- **Function**: `validatePropertiesUpdate(updates: Record<string, unknown>)`
- **Rules**:
  - Must be an object
  - Sanitizes all string values
  - Preserves non-string values (numbers, booleans)
- **Use Case**: Validates property updates

#### 9. Text Sanitization
- **Function**: `sanitizeText(text: string)`
- **Rules**:
  - Escapes HTML special characters: `& < > " ' /`
  - Prevents XSS attacks
- **Use Case**: Used by all validation functions for text sanitization

## Integration Points

### 1. InputModal Component
- **Location**: `apps/studio-core/src/components/InputModal.tsx`
- **Validation**: `validateNodeLabel()` on form submit
- **Behavior**: Shows error and prevents submission if invalid

### 2. Modal Handlers
- **Location**: `apps/studio-core/src/handlers/modalHandlers.ts`
- **Validations**:
  - `createHandleModalConfirm`: Validates node labels and relation labels
  - `createHandleAdrConfirm`: Validates ADR data
- **Behavior**: Shows toast error and prevents action if invalid

### 3. Search Dialog
- **Location**: `apps/studio-core/src/components/SearchDialog.tsx`
- **Validation**: `validateSearchQuery()` before searching
- **Behavior**: Sanitizes query before use

### 4. Properties Updates
- **Location**: `apps/studio-core/src/utils/viewerUtils.ts`
- **Validation**: Basic object validation in `handlePropertiesUpdate()`
- **Behavior**: Validates data structure before processing

## Security Features

### XSS Prevention
- ✅ HTML tag removal in labels
- ✅ HTML character escaping in all text inputs
- ✅ Sanitization before rendering

### Input Length Limits
- ✅ Node IDs: 1-100 characters
- ✅ Labels: 1-200 characters
- ✅ Search queries: 0-200 characters
- ✅ DSL input: 10MB maximum

### Type Validation
- ✅ Node type validation (only valid types allowed)
- ✅ ADR status validation (only valid statuses allowed)
- ✅ URL protocol validation (only http/https)

### Data Integrity
- ✅ Required field validation
- ✅ Format validation (node IDs must match pattern)
- ✅ Structure validation (objects, arrays)

## Error Handling

All validation functions return `ValidationResult`:
```typescript
interface ValidationResult {
  isValid: boolean;
  error?: string;
  sanitized?: string;
}
```

- **isValid**: Whether the input is valid
- **error**: Human-readable error message (if invalid)
- **sanitized**: Sanitized version of the input (if valid)

## Usage Examples

### Node Label Validation
```typescript
const validation = validateNodeLabel(userInput);
if (!validation.isValid) {
  showError(validation.error);
  return;
}
const sanitizedLabel = validation.sanitized;
```

### XSS Prevention
```typescript
const safeText = sanitizeText(userInput);
// safeText is now safe to render
```

### ADR Validation
```typescript
const validation = validateAdrData(adrData);
if (!validation.isValid) {
  showError(validation.error);
  return;
}
const sanitizedData = JSON.parse(validation.sanitized);
```

## Best Practices

1. **Always Validate**: Validate all user inputs before processing
2. **Sanitize Before Render**: Always sanitize before rendering user content
3. **Show Clear Errors**: Display validation errors to users
4. **Fail Fast**: Validate early, fail fast
5. **Log Validation Failures**: Log validation failures for security monitoring

## Future Enhancements

- [ ] Rate limiting for validation attempts
- [ ] More sophisticated DSL validation
- [ ] Content Security Policy (CSP) headers
- [ ] Input validation for file uploads (if added)
- [ ] Validation for export formats




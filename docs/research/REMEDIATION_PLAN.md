# Code Robustness Remediation Plan

**Status:** 🟡 In Progress  
**Created:** January 2025  
**Related Documents:** 
- [Research Report](./CODE_ROBUSTNESS_RESEARCH.md)
- [Executive Summary](./README.md)
- [Code Review Checklist](./CODE_REVIEW_CHECKLIST.md)

---

## Overview

This remediation plan provides a structured, actionable approach to address all identified weaknesses in the Sruja codebase. Tasks are prioritized by risk and impact, with clear assignments, timelines, and success criteria.

### Objectives

1. **Eliminate all compilation errors** within 2 weeks
2. **Remove production panics** (unwrap/expect) within 4 weeks
3. **Complete all broken features** within 6 weeks
4. **Achieve production-ready code quality** within 12 weeks

### Success Metrics

- Zero compilation errors across all crates
- <5 instances of `.unwrap()` in production Rust code
- Zero `any` types in TypeScript
- Zero `console.*` statements in production React code
- >80% test coverage on critical paths
- All documented features functional

---

## Phase 1: Critical Fixes (Weeks 1-2)

**Goal:** Eliminate all blockers preventing basic functionality

### 1.1 Fix Rust Compilation Errors

**Priority:** P0 - Critical  
**Team:** Rust Backend  
**Owner:** TBD  
**Effort:** 8 days

#### Task 1.1.1: Complete WASM Model-to-DSL Conversion
**File:** `sruja-wasm/src/lib.rs`  
**Current State:** Incomplete implementation, stub returning basic DSL  
**Issue:** Users cannot convert models back to DSL format in browser interface

**Implementation Steps:**

1. Analyze `SrujaModelDump` structure from `@sruja/shared`
```typescript
// Required conversion mapping:
// - elements: Object of ElementDump
// - relations: Array of RelationDump  
// - views: Object of ViewDump
// - specification: SpecificationDump
```

2. Implement DSL generation for elements:
```rust
// Pseudocode structure
fn element_to_dsl(element: &ElementDump) -> String {
    let mut dsl = String::new();
    
    // Basic structure: kind "Title" {
    dsl.push_str(&format!("{} \"{}\"", element.kind, element.title));
    
    // Optional description
    if let Some(desc) = &element.description {
        dsl.push_str(&format!("\n    description \"{}\"", desc));
    }
    
    // Optional technology
    if let Some(tech) = &element.technology {
        dsl.push_str(&format!("\n    technology \"{}\"", tech));
    }
    
    // Tags
    if !element.tags.is_empty() {
        dsl.push_str(&format!("\n    tags {}", 
            element.tags.iter().map(|t| format!("\"{}\"", t))
                .collect::<Vec<_>>().join(", ")));
    }
    
    // Metadata
    if !element.metadata.is_empty() {
        for (key, value) in &element.metadata {
            dsl.push_str(&format!("\n    metadata {} = \"{}\"", key, value));
        }
    }
    
    dsl.push_str("\n}");
    dsl
}
```

3. Implement DSL generation for relations:
```rust
fn relation_to_dsl(relation: &RelationDump) -> String {
    let mut dsl = String::new();
    
    // Format: source -> target "Label"
    dsl.push_str(&format!("{} -> {}", 
        relation.source.as_string(),
        relation.target.as_string()));
    
    if let Some(label) = &relation.title {
        dsl.push_str(&format!(" \"{}\"", label));
    }
    
    // Optional description
    if let Some(desc) = &relation.description {
        dsl.push_str(&format!(" {\n    description \"{}\"\n}", desc));
    }
    
    dsl
}
```

4. Add unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_element_conversion() {
        let model = json!({
            "elements": {
                "MySystem": {
                    "id": "MySystem",
                    "kind": "system",
                    "title": "My System",
                    "description": "A test system"
                }
            },
            "relations": [],
            "views": {}
        });

        let result = sruja_model_to_dsl(&model.to_string());
        assert!(result.is_ok());
        
        let dsl = result.unwrap();
        assert!(dsl.contains("system \"My System\""));
        assert!(dsl.contains("description \"A test system\""));
    }
    
    #[test]
    fn test_relation_conversion() {
        let model = json!({
            "elements": {},
            "relations": [{
                "id": "rel1",
                "source": "SystemA",
                "target": "SystemB",
                "title": "calls"
            }],
            "views": {}
        });

        let result = sruja_model_to_dsl(&model.to_string());
        assert!(result.is_ok());
        
        let dsl = result.unwrap();
        assert!(dsl.contains("SystemA -> SystemB \"calls\""));
    }
}
```

5. Update WASM bindings if needed

**Testing:**
- Unit tests for element conversion
- Unit tests for relation conversion
- Integration test with full model
- Browser integration test

**Success Criteria:**
- ✅ All tests pass
- ✅ Model-to-DSL conversion produces valid DSL
- ✅ Round-trip conversion works (DSL → Model → DSL)
- ✅ Zero compilation errors in sruja-wasm crate

**Dependencies:** None

**Risks & Mitigations:**
- Risk: Complex nested elements may not convert correctly
  - Mitigation: Implement recursion for nested structures, test thoroughly
- Risk: Metadata format may not match DSL expectations
  - Mitigation: Document metadata format, provide examples

---

#### Task 1.1.2: Implement LSP Code Actions
**File:** `sruja-lsp/src/server.rs` (Line 360)  
**Current State:** Returns `Ok(None)` - stub implementation  
**Issue:** No quick fixes or refactorings available in IDE

**Implementation Steps:**

1. Define code action types:
```rust
#[derive(Debug, Clone)]
pub enum SrujaCodeAction {
    FixSyntaxError { 
        range: Range, 
        replacement: String,
        message: String 
    },
    RenameElement { 
        old_name: String, 
        new_name: String 
    },
    AddDescription { 
        element_id: String 
    },
    ExtractRelation {
        from: String,
        to: String,
        label: Option<String>
    },
}
```

2. Implement code action provider:
```rust
async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
    let uri = &params.text_document.uri;
    let diagnostics = &params.context.diagnostics;
    
    // Collect relevant diagnostics
    let actions: Vec<CodeAction> = diagnostics
        .iter()
        .filter_map(|diag| self.create_fix_for_diagnostic(uri, diag))
        .flatten()
        .collect();
    
    if actions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(CodeActionResponse::Actions(actions)))
    }
}

fn create_fix_for_diagnostic(
    &self, 
    uri: &Url, 
    diag: &Diagnostic
) -> Option<Vec<CodeAction>> {
    match diag.code.as_ref().map(|c| c.as_str()) {
        Some("undefined-element") => self.create_element_creation_fix(diag),
        Some("missing-description") => self.create_add_description_fix(diag),
        Some("invalid-syntax") => self.create_syntax_fix(diag),
        _ => None,
    }
}
```

3. Implement specific code actions:
```rust
fn create_add_description_fix(&self, diag: &Diagnostic) -> Option<Vec<CodeAction>> {
    let element_id = extract_element_id_from_diagnostic(diag)?;
    
    let action = CodeAction {
        title: "Add description".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diag.clone()]),
        edit: Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![
                TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: diag.range.start.uri.clone(),
                        version: None,
                    },
                    edits: vec![TextEdit {
                        range: find_element_body_range(&element_id)?,
                        new_text: format!("\n    description \"TODO: Add description for {}\"", element_id),
                    }],
                }
            ])),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    Some(vec![action])
}
```

4. Add support for rename refactorings:
```rust
async fn prepare_rename(
    &self, 
    params: TextDocumentPositionParams
) -> Result<Option<PrepareRenameResponse>> {
    let text = self.documents.get(&params.text_document.uri)?;
    let position = params.position;
    
    // Find element at position
    if let Some(element) = self.find_element_at_position(text, position) {
        Ok(Some(PrepareRenameResponse::DefaultBehavior {
            range: element.name_range,
            placeholder: element.name,
        }))
    } else {
        Ok(None)
    }
}
```

5. Add tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_code_action_for_missing_description() {
        let server = setup_test_server();
        
        let code = r#"
system MySystem
"#;
        
        server.open_document("test.sruja", code).await;
        
        let diagnostic = Diagnostic {
            range: Range {
                start: Position { line: 1, character: 0 },
                end: Position { line: 1, character: 16 },
            },
            code: Some(NumberOrString::String("missing-description".to_string())),
            ..Default::default()
        };
        
        let actions = server.create_fix_for_diagnostic(&Url::from_file_path("test.sruja").unwrap(), &diagnostic);
        assert!(actions.is_some());
        
        let action_list = actions.unwrap();
        assert_eq!(action_list.len(), 1);
        assert_eq!(action_list[0].title, "Add description");
    }
}
```

**Testing:**
- Unit tests for each code action type
- Integration tests with language server
- Manual testing in VS Code extension
- Test with real DSL files

**Success Criteria:**
- ✅ Code actions return correctly for common diagnostics
- ✅ Text edits apply correctly
- ✅ No regressions in existing LSP functionality
- ✅ Zero compilation errors in sruja-lsp crate

**Dependencies:** None

**Risks & Mitigations:**
- Risk: Code actions may create invalid DSL
  - Mitigation: Validate all generated DSL syntax
- Risk: Range calculations may be off
  - Mitigation: Extensive testing with various file formats

---

#### Task 1.1.3: Resolve Other Compilation Errors
**Files:** Various  
**Current State:** 5 total errors across crates  
**Effort:** 2 days

**Known Errors:**
1. `sruja-lsp/src/diagnostics.rs` - 2 warnings
2. `sruja-lsp/src/server.rs` - 1 error, 1 warning (addressed in 1.1.2)

**Implementation Steps:**

1. Review all compilation errors with `cargo check --workspace`
2. Fix type mismatches
3. Resolve missing trait implementations
4. Address unused variable warnings
5. Fix clippy warnings

**Success Criteria:**
- ✅ `cargo check --workspace` completes with zero errors
- ✅ `cargo clippy --workspace` has zero warnings (or documented suppressions)

---

### 1.2 Remove Production Panics

**Priority:** P0 - Critical  
**Team:** Rust Backend  
**Owner:** TBD  
**Effort:** 5 days

#### Task 1.2.1: Replace .unwrap() in CLI Commands
**Files:** 
- `sruja-cli/src/commands.rs` (Lines 180-184, 249-253, 268)
- `sruja-cli/src/modules/file_operations.rs` (Test functions)

**Current Issue:** Using `.unwrap()` and `.expect()` that could panic

**Implementation Steps:**

1. Fix title extraction in list command:
```rust
// BEFORE (Line 180-184)
let title = elem
    .assignment
    .title
    .clone()
    .unwrap_or_else(|| elem.assignment.name.clone());

// AFTER
let title = elem.assignment.title.as_deref().unwrap_or(&elem.assignment.name);
```

2. Fix file path defaults:
```rust
// BEFORE (Line 268)
let file_path = file.unwrap_or("architecture.sruja");

// AFTER
let file_path = match file {
    Some(path) => path.to_string(),
    None => {
        // Check for default file
        if Path::new("architecture.sruja").exists() {
            "architecture.sruja".to_string()
        } else {
            return Err(CliError::FileNotFound(
                "No file specified and architecture.sruja not found".to_string()
            ));
        }
    }
};
```

3. Fix environment variable fallback:
```rust
// BEFORE (Line 620-624)
author = std::env::var("USER")
    .or_else(|_| std::env::var("USERNAME"))
    .unwrap_or_else(|_| "Unknown".to_string()),

// AFTER
author = std::env::var("USER")
    .or_else(|_| std::env::var("USERNAME"))
    .map_err(|_| CliError::ConfigError(
        "Could not determine author. Please set USER or USERNAME environment variable".to_string()
    ))?,
```

4. Update error type:
```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    // ... other variants
}
```

5. Update test functions to not use unwrap:
```rust
// BEFORE
File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

// AFTER
File::create(&file_path)
    .and_then(|mut f| f.write_all(content.as_bytes()))
    .expect("Failed to create test file");
```

**Testing:**
- All existing tests pass
- Add error case tests
- Test with missing files
- Test with invalid environment

**Success Criteria:**
- ✅ Zero `.unwrap()` calls in production CLI code
- ✅ All error cases handled gracefully
- ✅ User-friendly error messages
- ✅ No regressions in CLI functionality

---

#### Task 1.2.2: Replace .unwrap() in Parser
**File:** `sruja-language/src/parser.rs`  
**Current Issue:** Multiple `.unwrap()` calls that could panic during parsing

**Implementation Steps:**

1. Audit all `.unwrap()` usage in parser
2. Replace with proper error handling:
```rust
// BEFORE
let name = identifier.unwrap();

// AFTER
let name = identifier.ok_or_else(|| nom::Err::Error(nom::error::Error {
    code: nom::error::ErrorKind::Tag,
    input: input,
}))?;
```

3. Add custom error type for parser:
```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Expected {expected} at line {line}, column {column}")]
    Expected { expected: String, line: usize, column: usize },
    
    #[error("Unexpected token: {token}")]
    UnexpectedToken { token: String, line: usize, column: usize },
    
    #[error("Incomplete input")]
    Incomplete,
}
```

**Testing:**
- Add parser error tests
- Test with malformed input
- Verify error messages are helpful

**Success Criteria:**
- ✅ Zero `.unwrap()` calls in parser
- ✅ All parse errors return results
- ✅ Helpful error messages with line/column

---

### 1.3 Fix Critical React Features

**Priority:** P0 - Critical  
**Team:** Frontend  
**Owner:** TBD  
**Effort:** 5 days

#### Task 1.3.1: Implement Delete Functionality
**File:** `apps/designer/src/components/Details/UnifiedDetailsList.tsx`  
**Current Issue:** Delete logic is stubbed with console warning

**Implementation Steps:**

1. Understand `updateArchitecture` pattern:
```typescript
// The update function should modify the SrujaModelDump
const handleDelete = async () => {
  if (!deleteItem) return;

  await updateArchitecture((arch) => {
    const updatedElements = { ...arch.elements };
    const updatedRelations = arch.relations.filter(
      (r) => r.source !== deleteItem.id && r.target !== deleteItem.id
    );

    // Remove element
    delete updatedElements[deleteItem.id];

    return {
      ...arch,
      elements: updatedElements,
      relations: updatedRelations,
    };
  });
};
```

2. Handle cascade deletion of children:
```typescript
const collectChildren = (elements: SrujaModelDump['elements'], parentId: string): string[] => {
  const children: string[] = [];
  
  for (const [id, elem] of Object.entries(elements)) {
    if (elem.parent === parentId) {
      children.push(id);
      children.push(...collectChildren(elements, id));
    }
  }
  
  return children;
};

const handleDelete = async () => {
  if (!deleteItem) return;

  await updateArchitecture((arch) => {
    const elementsToDelete = [deleteItem.id, ...collectChildren(arch.elements, deleteItem.id)];
    
    const updatedElements = { ...arch.elements };
    elementsToDelete.forEach(id => delete updatedElements[id]);
    
    const updatedRelations = arch.relations.filter(
      (r) => !elementsToDelete.includes(r.source) && !elementsToDelete.includes(r.target)
    );

    return {
      ...arch,
      elements: updatedElements,
      relations: updatedRelations,
    };
  });
};
```

3. Add confirmation dialog:
```typescript
const confirmDelete = async () => {
  const confirmed = await confirm(
    `Are you sure you want to delete "${deleteItem.title}"?` +
    (hasChildren ? " This will also delete all children." : "")
  );
  
  if (confirmed) {
    await handleDelete();
    // Refresh details panel
    onRefresh?.();
    // Show success toast
    showToast("Element deleted successfully", "success");
  }
};
```

4. Add error handling:
```typescript
try {
  await handleDelete();
  showToast("Element deleted successfully", "success");
} catch (error) {
  logger.error("Failed to delete element", error);
  showToast(
    "Failed to delete element. Please try again.",
    "error"
  );
}
```

5. Add tests:
```typescript
import { renderHook, act } from '@testing-library/react';
import { useArchitectureStore } from '@/stores/architectureStore';

describe('delete element', () => {
  test('should delete element and its children', async () => {
    const initialModel = {
      elements: {
        'System1': { id: 'System1', kind: 'system', title: 'System 1' },
        'Container1': { id: 'Container1', kind: 'container', title: 'Container 1', parent: 'System1' },
      },
      relations: [],
      views: {}
    };

    const { result } = renderHook(() => useArchitectureStore());
    
    await act(async () => {
      await result.current.loadFromModel(initialModel);
      await result.current.deleteNodes(['System1']);
    });

    expect(result.current.model?.elements['System1']).toBeUndefined();
    expect(result.current.model?.elements['Container1']).toBeUndefined();
  });
});
```

**Testing:**
- Unit tests for delete logic
- Integration tests with architecture store
- Manual testing in UI
- Test cascade deletion
- Test with relations

**Success Criteria:**
- ✅ Delete functionality works for single element
- ✅ Cascade deletion works for nested elements
- ✅ Relations involving deleted elements are removed
- ✅ User confirmation dialog shows correct message
- ✅ Success/error toasts display correctly
- ✅ No regressions in existing functionality

---

#### Task 1.3.2: Connect Edit Handlers
**File:** `apps/designer/src/components/Panels/NavigationPanel.tsx`  
**Current Issue:** Edit button has empty handler

**Implementation Steps:**

1. Implement edit handler that opens inspector:
```typescript
const handleEdit = (elementId: string) => {
  // Select the element
  setSelectedNodeId(elementId);
  
  // Ensure inspector is visible
  if (!isInspectorVisible) {
    toggleInspector();
  }
  
  // Switch to properties tab in inspector
  setActiveTab('properties');
};
```

2. Update NavigationPanel:
```typescript
<Button
  variant="ghost"
  size="sm"
  onClick={() => handleEdit(container.id)}
  isEditMode={!!isEditMode()}
>
  Edit
</Button>
```

3. Add prop for setSelectedNodeId if not already available:
```typescript
interface NavigationPanelProps {
  onClose?: () => void;
  setSelectedNodeId?: (id: string | null) => void;
}
```

**Testing:**
- Click edit button → element selected
- Inspector opens if closed
- Inspector shows properties tab
- Test with various element types

**Success Criteria:**
- ✅ Edit button selects element
- ✅ Inspector opens automatically
- ✅ Correct tab is selected
- ✅ Works for all element types

---

#### Task 1.3.3: Implement Metadata Deletion
**File:** `apps/designer/src/components/Panels/OverviewPanel.tsx`  
**Current Issue:** Delete functions are stubbed with warnings

**Implementation Steps:**

1. Implement metadata deletion:
```typescript
const handleDeleteMetadata = async (index: number, key: string) => {
  try {
    await updateArchitecture((arch) => {
      const updatedMetadata = [...(arch.sruja?.metadata || [])];
      updatedMetadata.splice(index, 1);
      
      return {
        ...arch,
        sruja: {
          ...arch.sruja,
          metadata: updatedMetadata,
        }
      };
    });
    
    showToast("Metadata deleted successfully", "success");
  } catch (error) {
    logger.error("Failed to delete metadata", error);
    showToast("Failed to delete metadata", "error");
  }
};
```

2. Implement constraint deletion:
```typescript
const handleDeleteConstraint = async (index: number, key: string) => {
  try {
    await updateArchitecture((arch) => {
      const updatedConstraints = [...(arch.sruja?.constraints || [])];
      updatedConstraints.splice(index, 1);
      
      return {
        ...arch,
        sruja: {
          ...arch.sruja,
          constraints: updatedConstraints,
        }
      };
    });
    
    showToast("Constraint deleted successfully", "success");
  } catch (error) {
    logger.error("Failed to delete constraint", error);
    showToast("Failed to delete constraint", "error");
  }
};
```

3. Implement convention deletion:
```typescript
const handleDeleteConvention = async (index: number, key: string) => {
  try {
    await updateArchitecture((arch) => {
      const updatedConventions = [...(arch.sruja?.conventions || [])];
      updatedConventions.splice(index, 1);
      
      return {
        ...arch,
        sruja: {
          ...arch.sruja,
          conventions: updatedConventions,
        }
      };
    });
    
    showToast("Convention deleted successfully", "success");
  } catch (error) {
    logger.error("Failed to delete convention", error);
    showToast("Failed to delete convention", "error");
  }
};
```

**Testing:**
- Delete each type of metadata
- Verify it's removed from model
- Verify UI updates correctly
- Test error handling

**Success Criteria:**
- ✅ Metadata deletion works
- ✅ Constraint deletion works
- ✅ Convention deletion works
- ✅ UI updates immediately
- ✅ Success/error toasts display

---

## Phase 2: High Priority Fixes (Weeks 3-4)

**Goal:** Complete feature implementations and improve type safety

### 2.1 Implement Parser Position Tracking

**Priority:** P1 - High  
**Team:** Rust Backend  
**Owner:** TBD  
**Effort:** 4 days

#### Implementation Steps:

1. Track position during parsing:
```rust
// Modify parser to track position
fn parse_element_def(input: &str) -> IResult<&str, ElementDef> {
    let start_pos = input.as_ptr() as usize;
    
    let (input, kind) = parse_element_kind(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, title) = parse_string_literal(input)?;
    let (input, body) = opt(parse_element_body)(input)?;
    
    let end_pos = input.as_ptr() as usize;
    let consumed = &input[start_pos - input.as_ptr() as usize..end_pos];
    
    // Calculate line and column
    let lines: Vec<&str> = consumed.lines().collect();
    let line = lines.len();
    let column = lines.last().map(|l| l.len()).unwrap_or(0);
    
    Ok((input, ElementDef {
        location: SourceLocation::new("file.sruja".to_string(), line, column),
        assignment: ElementAssignment {
            location: SourceLocation::new("file.sruja".to_string(), line, column),
            name,
            kind,
            sub_kind: None,
            title,
            tag_refs: vec![],
            body,
        },
    }))
}
```

2. Update nom combinator wrapper:
```rust
fn with_position<F>(parser: F, file: String) -> impl Fn(&str) -> IResult<&str, Spanned<&str>>
where
    F: Fn(&str) -> IResult<&str, &str>,
{
    move |input| {
        let start = input.as_ptr();
        let (remaining, result) = parser(input)?;
        let end = remaining.as_ptr();
        
        let consumed = unsafe {
            let len = (start as usize) - (end as usize);
            std::slice::from_raw_parts(start, len)
        };
        
        let line = consumed.iter().filter(|&&c| c == b'\n').count() + 1;
        let column = consumed.iter().rev().take_while(|&&c| c != b'\n').count();
        
        Ok((remaining, (result, SourceLocation::new(file.clone(), line, column))))
    }
}
```

**Testing:**
- Verify position tracking is accurate
- Test with multi-line elements
- Test error reporting with positions

**Success Criteria:**
- ✅ All AST nodes have accurate locations
- ✅ Error messages show correct line/column
- ✅ Diagnostics clickable in editor

---

### 2.2 Complete DSL Formatter

**Priority:** P1 - High  
**Team:** Rust Backend  
**Owner:** TBD  
**Effort:** 3 days

#### Implementation Steps:

1. Implement DSL printer:
```rust
// sruja-export/src/dsl/printer.rs
use sruja_language::*;

pub struct DslPrinter {
    indent_size: usize,
}

impl DslPrinter {
    pub fn new() -> Self {
        Self { indent_size: 4 }
    }
    
    pub fn print(&self, program: &Program) -> String {
        let mut output = String::new();
        
        // Print metadata
        if let Some(metadata) = &program.metadata {
            self.print_metadata(&mut output, metadata);
            output.push('\n');
        }
        
        // Print elements
        for item in &program.items {
            self.print_top_level_item(&mut output, item);
            output.push('\n');
        }
        
        // Print relations
        for relation in &program.relations {
            self.print_relation(&mut output, relation);
            output.push('\n');
        }
        
        output.trim().to_string()
    }
    
    fn print_element(&self, output: &mut String, element: &ElementDef, indent: usize) {
        let prefix = " ".repeat(indent);
        
        // Print element declaration
        output.push_str(&prefix);
        output.push_str(&element.assignment.kind.to_string());
        output.push(' ');
        output.push_str(&element.assignment.name);
        
        // Print title if exists
        if let Some(title) = &element.assignment.title {
            output.push_str(&format!(" \"{}\"", title));
        }
        
        // Print body if exists
        if let Some(body) = &element.assignment.body {
            output.push_str(" {\n");
            self.print_element_body(output, body, indent + self.indent_size);
            output.push_str(&prefix);
            output.push_str("}\n");
        } else {
            output.push('\n');
        }
    }
    
    fn print_element_body(&self, output: &mut String, body: &ElementBody, indent: usize) {
        let prefix = " ".repeat(indent);
        
        // Description
        if let Some(description) = &body.description {
            output.push_str(&prefix);
            output.push_str(&format!("description \"{}\"\n", description));
        }
        
        // Technology
        if let Some(technology) = &body.technology {
            output.push_str(&prefix);
            output.push_str(&format!("technology \"{}\"\n", technology));
        }
        
        // Tags
        if !body.tags.is_empty() {
            output.push_str(&prefix);
            output.push_str(&format!("tags {}\n", 
                body.tags.iter()
                    .map(|t| format!("\"{}\"", t))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        
        // Nested elements
        for element in &body.elements {
            self.print_element(output, element, indent);
        }
        
        // Relations within body
        for relation in &body.relations {
            output.push_str(&prefix);
            self.print_relation_inline(output, relation);
            output.push('\n');
        }
    }
    
    fn print_relation(&self, output: &mut String, relation: &Relation) {
        self.print_relation_inline(output, relation);
        output.push('\n');
    }
    
    fn print_relation_inline(&self, output: &mut String, relation: &Relation) {
        output.push_str(&relation.from.as_string());
        output.push_str(" -> ");
        output.push_str(&relation.to.as_string());
        
        if let Some(label) = &relation.label {
            output.push_str(&format!(" \"{}\"", label));
        }
    }
}
```

2. Implement CLI formatter command:
```rust
// sruja-cli/src/commands.rs
pub async fn fmt(file: &str) -> Result<(), CliError> {
    let content = fs::read_to_string(file)?;
    let parser = Parser::new(file.to_string());
    
    let program = parser.parse(&content)
        .map_err(|diags| CliError::ParseError {
            file: file.to_string(),
            diagnostics: diags,
        })?;
    
    let printer = DslPrinter::new();
    let formatted = printer.print(&program);
    
    fs::write(file, formatted)?;
    
    println!("Formatted: {}", file);
    Ok(())
}
```

**Testing:**
- Test with simple DSL
- Test with nested structures
- Test with metadata
- Verify round-trip (format → parse → format is idempotent)

**Success Criteria:**
- ✅ Formatter produces consistent output
- ✅ Preserves semantics
- ✅ Improves readability
- ✅ Round-trip is idempotent

---

### 2.3 Eliminate TypeScript `any` Types

**Priority:** P1 - High  
**Team:** Frontend  
**Owner:** TBD  
**Effort:** 3 days

#### Task 2.3.1: Fix Canvas View Type
**File:** `apps/designer/src/components/SrujaCanvas/SrujaCanvas.tsx` (Line 908)

```typescript
// BEFORE
const tryGetPositions = (
  view: any
): Record<string, { x: number; y: number }> | undefined => {

// AFTER
interface ViewPositions {
  [key: string]: { x: number; y: number };
}

const tryGetPositions = (
  view: ViewPositions | undefined
): Record<string, { x: number; y: number }> | undefined => {
  if (!view) return undefined;
  
  // Type guard to ensure structure
  const positions: Record<string, { x: number; y: number }> = {};
  
  for (const [key, pos] of Object.entries(view)) {
    if (typeof pos === 'object' && pos !== null && 
        'x' in pos && 'y' in pos &&
        typeof pos.x === 'number' && typeof pos.y === 'number') {
      positions[key] = { x: pos.x, y: pos.y };
    }
  }
  
  return positions;
};
```

#### Task 2.3.2: Fix Type Suppressions
**File:** `apps/designer/src/components/SrujaCanvas/SrujaCanvas.tsx` (Line 582)

```typescript
// BEFORE
metadata: {
  // @ts-expect-error - position is not in ElementDump type but needed for layout hint
  position: { x: position.x + index * 40, y: position.y + index * 40 },
}

// AFTER - Create proper type extension
interface ElementDumpWithPosition extends ElementDump {
  position?: { x: number; y: number };
}

const elementDump: ElementDumpWithPosition = {
  ...element,
  position: { x: position.x + index * 40, y: position.y + index * 40 },
};
```

Or better, update the shared types:
```typescript
// In @sruja/shared types
export interface ElementDump {
  id: string;
  kind: string;
  title: string;
  description?: string;
  technology?: string;
  tags: string[];
  links: string[];
  metadata: Record<string, string>;
  style?: Record<string, unknown>;
  parent?: string;
  layout?: {  // Add this
    position?: { x: number; y: number };
  };
}
```

**Testing:**
- TypeScript compilation passes
- No more `any` types
- No more type suppressions
- Runtime behavior unchanged

**Success Criteria:**
- ✅ Zero `any` types
- ✅ Zero `@ts-ignore` or `@ts-expect-error`
- ✅ Strict TypeScript compilation
- ✅ Full type safety maintained

---

### 2.4 Remove Console Statements

**Priority:** P1 - High  
**Team:** Frontend  
**Owner:** TBD  
**Effort:** 2 days

#### Implementation Steps:

1. Set up proper logger:
```typescript
// apps/designer/src/utils/logger.ts
import * as Sentry from '@sentry/react';

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

class Logger {
  private minLevel: LogLevel;
  
  constructor() {
    this.minLevel = import.meta.env.PROD ? LogLevel.INFO : LogLevel.DEBUG;
  }
  
  private shouldLog(level: LogLevel): boolean {
    return level >= this.minLevel;
  }
  
  debug(message: string, context?: Record<string, unknown>) {
    if (this.shouldLog(LogLevel.DEBUG)) {
      console.debug(`[DEBUG] ${message}`, context);
    }
  }
  
  info(message: string, context?: Record<string, unknown>) {
    if (this.shouldLog(LogLevel.INFO)) {
      console.info(`[INFO] ${message}`, context);
    }
  }
  
  warn(message: string, context?: Record<string, unknown>) {
    if (this.shouldLog(LogLevel.WARN)) {
      console.warn(`[WARN] ${message}`, context);
      Sentry.captureMessage(message, 'warning', { extra: context });
    }
  }
  
  error(message: string, error?: Error | Record<string, unknown>, context?: Record<string, unknown>) {
    if (this.shouldLog(LogLevel.ERROR)) {
      console.error(`[ERROR] ${message}`, error, context);
      Sentry.captureException(error instanceof Error ? error : new Error(message), {
        extra: context,
      });
    }
  }
}

export const logger = new Logger();
```

2. Replace all console statements:
```typescript
// BEFORE
console.log("[SrujaCanvas] Clearing stale activeViewId", { activeViewId });

// AFTER
logger.debug("Clearing stale activeViewId", { activeViewId });
```

3. Add ESLint rule:
```json
{
  "rules": {
    "no-console": ["error", { "allow": ["error", "warn"] }]
  }
}
```

4. Configure build to strip debug logs:
```typescript
// vite.config.ts
import defineConfig from 'vite';

export default defineConfig({
  define: {
    __LOGGER_LEVEL__: JSON.stringify(
      import.meta.env.PROD ? 1 : 0 // INFO level in production, DEBUG in dev
    ),
  },
});
```

**Testing:**
- All logs use logger
- Logs appear in dev
- Logs controlled in production
- Errors sent to Sentry

**Success Criteria:**
- ✅ Zero `console.log` statements
- ✅ Zero `console.warn` statements (use logger.warn)
- ✅ Only `console.error` for actual errors
- ✅ Logger configured for environment

---

## Phase 3: Medium Priority (Weeks 5-6)

**Goal:** Complete remaining features and improve architecture

### 3.1 Complete Feature Implementations

#### 3.1.1 View Conversion
**File:** `sruja-export/src/json/exporter.rs`  
**Effort:** 2 days

```rust
fn convert_views_from_program(&self, dump: &mut SrujaModelDump, program: &Program) {
    let mut views: HashMap<String, ViewDump> = HashMap::new();
    
    // Process ViewDef items
    for item in &program.items {
        if let TopLevelItem::View(view_def) = item {
            let mut view = ViewDump {
                id: view_def.name.clone(),
                title: view_def.title.clone().unwrap_or_else(|| view_def.name.clone()),
                description: view_def.description.clone(),
                kind: view_def.kind.map(|k| k.to_string()).unwrap_or_else(|| "system".to_string()),
                element_ids: vec![],
                include: vec![],
                exclude: vec![],
                styles: HashMap::new(),
            };
            
            // Process includes/excludes
            for rule in &view_def.rules {
                if rule.include {
                    view.include.push(rule.element_id.clone());
                } else {
                    view.exclude.push(rule.element_id.clone());
                }
            }
            
            views.insert(view_def.name.clone(), view);
        }
    }
    
    dump.views = views;
}
```

#### 3.1.2 Scenario Validation
**File:** `sruja-engine/src/rules/scenario_validation.rs`  
**Effort:** 1 day

```rust
// Add validation for inline scenarios
TopLevelItem::Element(elem) => {
    // Check for inline scenarios in element body
    if let Some(body) = &elem.assignment.body {
        for scenario in &body.scenarios {
            diags.extend(runner.validate_scenario(scenario, &elem.location));
        }
    }
}
```

#### 3.1.3 Metadata/Style Export
**File:** `sruja-export/src/json/exporter.rs`  
**Effort:** 1 day

```rust
// Extract links from metadata
links: body.links.iter().map(|l| l.clone()).collect(),

// Extract style
style: body.style.map(|s| StyleDump {
    color: s.color,
    shape: s.shape.map(|shape| shape.to_string()),
    opacity: s.opacity,
}),
```

---

### 3.2 Improve Error Handling

#### 3.2.1 Add User Feedback to Silent Catches
**Files:** Multiple React components  
**Effort:** 2 days

```typescript
// Governance components
catch (error) {
  setScoreCard(null);
  showToast(
    "Score calculation failed. Please check your DSL syntax.",
    "error"
  );
}

// DslPreview
catch (err) {
  showToast(
    "Failed to copy to clipboard. Please check browser permissions.",
    "error"
  );
}
```

#### 3.2.2 Add Error Boundaries
**Effort:** 1 day

```typescript
// Wrap major sections
<ErrorBoundary fallback={<ErrorFallback />}>
  <SrujaCanvas />
</ErrorBoundary>

<ErrorBoundary fallback={<ErrorFallback />}>
  <BuilderWizard />
</ErrorBoundary>
```

---

### 3.3 Fix FQN Resolution

**File:** `apps/designer/src/utils/fqnResolver.ts`  
**Effort:** 1 day

```typescript
function resolveFqnToNodeId(fqn: string, model: SrujaModelDump): string | null {
  // Direct match
  if (model.elements[fqn]) {
    return fqn;
  }
  
  // Check for collisions
  const matches = Object.keys(model.elements).filter(id => 
    id === fqn || id.endsWith('.' + fqn)
  );
  
  if (matches.length === 0) {
    return null;
  }
  
  if (matches.length === 1) {
    return matches[0];
  }
  
  // Collision - return null to force explicit selection
  console.warn(`Ambiguous FQN: ${fqn} matches ${matches.join(', ')}`);
  return null;
}
```

---

### 3.4 Remove Legacy Browser Support

**File:** `apps/designer/src/utils/shareService.ts`  
**Effort:** 0.5 day

```typescript
// Remove fallback, only support modern browsers
private generateShareId(): string {
  if (typeof crypto === "undefined" || !crypto.randomUUID) {
    throw new Error(
      "Browser does not support crypto.randomUUID. Please use a modern browser."
    );
  }
  return crypto.randomUUID();
}
```

---

## Phase 4: Long-term Improvements (Weeks 7-12)

**Goal:** Enhance architecture, testing, and documentation

### 4.1 Refactor Large Components

**File:** `apps/designer/src/components/SrujaCanvas/SrujaCanvas.tsx`  
**Effort:** 2 weeks

**Refactoring Strategy:**

1. Extract layout logic:
```typescript
// SrujaCanvas/useCanvasLayout.ts
export function useCanvasLayout(model: SrujaModelDump, activeViewId: string | null) {
  const [layout, setLayout] = useState<LayoutResult | null>(null);
  const [isComputing, setIsComputing] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  
  const computeLayout = useCallback(async () => {
    setIsComputing(true);
    setError(null);
    
    try {
      const result = await computeCanvasLayout(model, activeViewId);
      setLayout(result);
    } catch (err) {
      setError(err as Error);
    } finally {
      setIsComputing(false);
    }
  }, [model, activeViewId]);
  
  return { layout, isComputing, error, computeLayout };
}
```

2. Extract node management:
```typescript
// SrujaCanvas/useCanvasNodes.ts
export function useCanvasNodes(layout: LayoutResult | null) {
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  
  useEffect(() => {
    if (!layout) return;
    
    const { nodes: layoutNodes, edges: layoutEdges } = convertLayoutToReactFlow(layout);
    setNodes(layoutNodes);
    setEdges(layoutEdges);
  }, [layout]);
  
  const onNodesChange = useCallback((changes: NodeChange[]) => {
    setNodes((nds) => applyNodeChanges(changes, nds));
  }, []);
  
  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    setEdges((eds) => applyEdgeChanges(changes, eds));
  }, []);
  
  return { nodes, edges, onNodesChange, onEdgesChange };
}
```

3. Extract drag and drop:
```typescript
// SrujaCanvas/useCanvasDragDrop.ts
export function useCanvasDragDrop(canvasRef: RefObject<CanvasHandle>) {
  const onDrop = useCallback(async (event: React.DragEvent) => {
    event.preventDefault();
    
    const featureData = JSON.parse(event.dataTransfer.getData('feature'));
    const position = calculateDropPosition(event);
    
    await canvasRef.current?.addElementAt(featureData, position);
  }, [canvasRef]);
  
  const onDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);
  
  return { onDrop, onDragOver };
}
```

4. Refactor main component:
```typescript
export const SrujaCanvas: React.FC = () => {
  const { model, activeViewId } = useViewStore();
  const { layout, isComputing, error, computeLayout } = useCanvasLayout(model, activeViewId);
  const { nodes, edges, onNodesChange, onEdgesChange } = useCanvasNodes(layout);
  const { onDrop, onDragOver } = useCanvasDragDrop(canvasRef);
  
  // Simplified render
  return (
    <div className="canvas-wrapper">
      {isComputing && <LoadingSpinner />}
      {error && <ErrorMessage error={error} />}
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onDrop={onDrop}
        onDragOver={onDragOver}
      />
    </div>
  );
};
```

**Benefits:**
- Easier to test
- Better performance (memoization)
- Reusable hooks
- Clearer separation of concerns

---

### 4.2 Comprehensive Testing

**Goal:** >80% coverage on critical paths

**Test Plan:**

1. **Rust Unit Tests** (Target: 85% coverage)
   - Parser tests: All grammar rules
   - AST traversal tests
   - Validator tests: All rules
   - Exporter tests: All formats

2. **Rust Integration Tests** (Target: 70% coverage)
   - End-to-end CLI commands
   - LSP protocol handling
   - WASM bridge functionality

3. **React Component Tests** (Target: 80% coverage)
   - All major components tested
   - Custom hooks tested
   - User interactions tested

4. **E2E Tests** (Critical paths)
   - Create architecture from scratch
   - Edit elements and relations
   - Export to all formats
   - Import from DSL
   - Calculate governance score

**Implementation:**
```bash
# Rust coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --workspace

# React coverage
vitest run --coverage

# E2E tests
playwright test
```

---

### 4.3 Performance Optimization

**Goals:**
- Page load <3s for typical architecture
- Layout computation <500ms for medium diagrams
- WASM bundle <500KB gzipped

**Optimizations:**

1. **React Performance**
```typescript
// Memoize expensive components
export const CanvasNode = React.memo<NodeData>(({ data }) => {
  // ...
}, (prev, next) => prev.data.id === next.data.id && prev.data.updated === next.data.updated);

// Memoize expensive calculations
const layoutData = useMemo(() => {
  return computeLayout(model, view);
}, [model, view]);

// Use useCallback for handlers
const handleClick = useCallback(() => {
  // ...
}, [dependencies]);
```

2. **WASM Optimization**
```rust
// Optimize for size
[profile.release]
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
strip = true

// Remove debug symbols
[profile.release.package."sruja-wasm"]
debug = false
```

3. **Code Splitting**
```typescript
// Lazy load heavy components
const SrujaCanvas = lazy(() => import('./components/SrujaCanvas'));
const BuilderWizard = lazy(() => import('./components/Wizard/BuilderWizard'));

// Route-based splitting
const Designer = lazy(() => import('./pages/Designer'));
const Documentation = lazy(() => import('./pages/Documentation'));
```

---

### 4.4 Documentation Overhaul

**Documentation Deliverables:**

1. **API Documentation**
   - All public Rust functions documented with examples
   - All React components with props and usage examples
   - Type definitions documented

2. **Architecture Documentation**
   - High-level architecture diagram
   - Data flow diagrams
   - Component relationship diagrams

3. **Developer Onboarding**
   - Quick start guide
   - Development environment setup
   - Testing guide
   - Contribution guidelines

4. **User Documentation**
   - DSL language reference
   - User manual
   - Tutorial and examples
   - Troubleshooting guide

**Implementation:**
```markdown
# API Documentation Example

## Element

Represents an architectural element in the system.

### Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| id | string | Yes | Unique identifier for the element |
| kind | ElementKind | Yes | Type of element (system, container, component, etc.) |
| title | string | Yes | Human-readable title |
| description | string | No | Detailed description of the element |

### Example

```typescript
const element: Element = {
  id: 'MySystem',
  kind: 'system',
  title: 'My System',
  description: 'A sample system'
};
```
```

---

## Implementation Timeline

### Week 1-2: Critical Fixes (P0)
- ✅ Day 1-8: Fix WASM compilation errors
- ✅ Day 9-10: Fix LSP compilation errors
- ✅ Day 11-12: Replace .unwrap() in CLI
- ✅ Day 13-14: Replace .unwrap() in parser
- ✅ Day 15-16: Implement delete functionality
- ✅ Day 17-18: Connect edit handlers
- ✅ Day 19-20: Implement metadata deletion

### Week 3-4: High Priority (P1)
- ✅ Day 21-24: Implement parser position tracking
- ✅ Day 25-27: Complete DSL formatter
- ✅ Day 28-30: Eliminate TypeScript any types
- ✅ Day 31-32: Remove console statements

### Week 5-6: Medium Priority (P2)
- ✅ Day 33-34: Complete view conversion
- ✅ Day 35: Complete scenario validation
- ✅ Day 36: Extract metadata/style in export
- ✅ Day 37-38: Improve error handling
- ✅ Day 39: Fix FQN resolution
- ✅ Day 40: Remove legacy browser support

### Week 7-8: Refactoring
- ✅ Day 41-54: Refactor SrujaCanvas component

### Week 9-10: Testing
- ✅ Day 55-60: Implement Rust unit tests
- ✅ Day 61-65: Implement React component tests
- ✅ Day 66-70: Implement E2E tests

### Week 11-12: Performance & Docs
- ✅ Day 71-74: Performance optimization
- ✅ Day 75-80: Documentation overhaul

---

## Risk Management

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| WASM conversion complexity | High | High | Incremental implementation, thorough testing |
| Parser position tracking complexity | Medium | High | Start with simple tracking, enhance incrementally |
| Canvas refactoring regressions | Medium | High | Extensive testing, feature flags |
| Test coverage targets not met | Medium | Medium | Focus on critical paths first, extend later |
| Documentation backlog | Low | Medium | Document as you go, prioritize user-facing docs |

### Rollback Procedures

1. **Feature Branch Strategy**
   - All work in feature branches
   - Main branch always stable
   - Cherry-pick fixes if needed

2. **Feature Flags**
   ```typescript
   const FLAGS = {
     NEW_DELETE_FUNCTIONALITY: import.meta.env.VITE_NEW_DELETE === 'true',
     IMPROVED_LAYOUT: import.meta.env.VITE_IMPROVED_LAYOUT === 'true',
   };
   ```

3. **Automated Revert**
   - Monitor error rates
   - Auto-revert on threshold breach
   - Alerts for sudden regressions

---

## Success Criteria Tracking

### Weekly Metrics

**Week 1-2 (Critical):**
- [ ] Zero compilation errors
- [ ] <10 instances of .unwrap() in production code
- [ ] Delete functionality working

**Week 3-4 (High Priority):**
- [ ] Parser position tracking complete
- [ ] DSL formatter working
- [ ] Zero `any` types
- [ ] Zero console statements

**Week 5-6 (Medium Priority):**
- [ ] All TODO/FIXME resolved
- [ ] Error handling improved
- [ ] FQN resolution complete

**Week 7-12 (Long-term):**
- [ ] Canvas refactored
- [ ] >80% test coverage
- [ ] Performance targets met
- [ ] Documentation complete

---

## Resources

### Team Assignments

| Role | Tasks | Effort |
|------|-------|--------|
| Rust Backend Lead | WASM fixes, Parser tracking, Formatter | 10 days |
| LSP Specialist | LSP code actions | 3 days |
| Frontend Lead | Delete functionality, Type safety | 8 days |
| Frontend Engineer | Error handling, Console removal | 4 days |
| Frontend Engineer | Canvas refactoring | 10 days |
| QA Engineer | Test implementation | 10 days |
| Technical Writer | Documentation | 8 days |

### Tools

- **Rust:** cargo, rustfmt, clippy, cargo-tarpaulin
- **TypeScript:** tsc, eslint, prettier, vitest
- **E2E:** playwright
- **CI/CD:** GitHub Actions
- **Monitoring:** Sentry

---

## Conclusion

This remediation plan provides a structured approach to addressing all identified weaknesses in the Sruja codebase. By following this plan, the team will achieve:

1. **Production-ready code** with zero critical issues
2. **Improved maintainability** through better type safety and documentation
3. **Enhanced user experience** with complete features and better error handling
4. **Long-term stability** through comprehensive testing and performance optimization

The plan is flexible and can be adjusted based on team capacity and business priorities, but the critical fixes (Phase 1) should be completed before considering any production deployment.

---

**Document Version:** 1.0  
**Last Updated:** January 2025  
**Next Review Date:** Weekly during Phase 1-2, then bi-weekly  
**Approvals:** Tech Lead, Engineering Manager, Product Owner
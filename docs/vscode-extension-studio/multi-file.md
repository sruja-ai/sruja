# Multi-File & Cross-File Handling

# 📌 Scope
VSCode Extension Studio behavior: two-way DSL↔diagram editing across multiple files using IR source locations; ELK layouting and editor updates occur inside the Webview.

# 🔀 Diagram Editor Multi-File
Understands references across `.sruja` files, imports, partitions, partial models.
IR compaction ensures:
```
Multiple DSL files → One IR → One diagram
```
Two-way binding modifies the correct file via source mapping.

# 🟧 Graph → IR Multi-File DSL Support
Kernel generates DSL patches using:
```
IRNode.location.file
```
Links graph edits to the right DSL file; resolves ambiguity; patches only in target file.

# 🟪 DSL Patch Generator Multi-File Support
- Pick correct file via `irNode.sourceLocation.file`
- Moves: remove from file A, add to file B
- Cross-file relations: insert in originating file or a global file (configurable)

# 🟣 Summary
Multi-file is handled consistently across editor, patch spec, and DSL generator using source locations and IR compaction.

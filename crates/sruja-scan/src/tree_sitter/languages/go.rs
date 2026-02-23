//! Go parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_go::LANGUAGE.into()).ok()?;

    let tree = parser.parse(content, None)?;
    let root = tree.root_node();

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut definitions = Vec::new();

    extract_from_node(&root, content, &mut imports, &mut exports, &mut definitions);

    Some(ParsedFile {
        name,
        path: path.to_string_lossy().to_string(),
        imports,
        exports,
        definitions,
    })
}

fn extract_from_node(
    node: &tree_sitter::Node,
    content: &str,
    imports: &mut Vec<String>,
    exports: &mut Vec<String>,
    definitions: &mut Vec<Definition>,
) {
    let kind = node.kind();

    match kind {
        "import_declaration" => {
            extract_import(node, content, imports);
        }
        "function_declaration" | "method_declaration" => {
            if let Some(name) = extract_function_name(node, content) {
                let is_exported = name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
                if is_exported {
                    exports.push(name);
                }
            }
        }
        "type_declaration" => {
            extract_type_declaration(node, content, exports, definitions);
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            extract_from_node(&child, content, imports, exports, definitions);
        }
    }
}

fn extract_import(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "import_spec" => {
                    if let Some(path) = extract_import_path(&child, content) {
                        imports.push(path);
                    }
                }
                "import_spec_list" => {
                    for j in 0..child.child_count() {
                        if let Some(spec) = child.child(j) {
                            if spec.kind() == "import_spec" {
                                if let Some(path) = extract_import_path(&spec, content) {
                                    imports.push(path);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn extract_import_path(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "interpreted_string_literal" {
                let text = child.utf8_text(content.as_bytes()).ok()?;
                return Some(text.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn extract_function_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
            if child.kind() == "field_identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_type_declaration(
    node: &tree_sitter::Node,
    content: &str,
    exports: &mut Vec<String>,
    definitions: &mut Vec<Definition>,
) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_spec" {
                for j in 0..child.child_count() {
                    if let Some(type_node) = child.child(j) {
                        match type_node.kind() {
                            "type_identifier" => {
                                if let Ok(name) = type_node.utf8_text(content.as_bytes()) {
                                    let is_exported = name
                                        .chars()
                                        .next()
                                        .map(|c| c.is_uppercase())
                                        .unwrap_or(false);
                                    definitions.push(Definition {
                                        name: name.to_string(),
                                        kind: DefinitionKind::Struct,
                                        line: node.start_position().row + 1,
                                    });
                                    if is_exported {
                                        exports.push(name.to_string());
                                    }
                                }
                            }
                            "struct_type" | "interface_type" => {
                                if j > 0 {
                                    if let Some(prev) = child.child(j - 1) {
                                        if prev.kind() == "type_identifier" {
                                            if let Ok(name) = prev.utf8_text(content.as_bytes()) {
                                                let is_exported = name
                                                    .chars()
                                                    .next()
                                                    .map(|c| c.is_uppercase())
                                                    .unwrap_or(false);
                                                definitions.push(Definition {
                                                    name: name.to_string(),
                                                    kind: if type_node.kind() == "interface_type" {
                                                        DefinitionKind::Interface
                                                    } else {
                                                        DefinitionKind::Struct
                                                    },
                                                    line: node.start_position().row + 1,
                                                });
                                                if is_exported {
                                                    exports.push(name.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_go_imports() {
        let code = r#"
package main

import (
    "fmt"
    "github.com/gin-gonic/gin"
)
"#;
        let result = parse(Path::new("main.go"), code).unwrap();
        assert!(result.imports.contains(&"fmt".to_string()));
        assert!(result.imports.iter().any(|i| i.contains("gin")));
    }

    #[test]
    fn test_parse_go_struct() {
        let code = r#"
package main

type User struct {
    ID   string
    Name string
}

func (u *User) GetID() string {
    return u.ID
}
"#;
        let result = parse(Path::new("user.go"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "User"));
        assert!(result.exports.contains(&"User".to_string()));
    }
}

//! Rust parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .ok()?;

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
        "use_declaration" => {
            extract_use(node, content, imports);
        }
        "function_item" => {
            if let Some(name) = extract_name(node, content) {
                let is_exported = is_public(node, content);
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
        "struct_item" => {
            if let Some(name) = extract_name(node, content) {
                let is_exported = is_public(node, content);
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Struct,
                    line: node.start_position().row + 1,
                });
                if is_exported {
                    exports.push(name);
                }
            }
        }
        "enum_item" => {
            if let Some(name) = extract_name(node, content) {
                let is_exported = is_public(node, content);
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
                if is_exported {
                    exports.push(name);
                }
            }
        }
        "trait_item" => {
            if let Some(name) = extract_name(node, content) {
                let is_exported = is_public(node, content);
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Interface,
                    line: node.start_position().row + 1,
                });
                if is_exported {
                    exports.push(name);
                }
            }
        }
        "const_item" | "static_item" => {
            if let Some(name) = extract_name(node, content) {
                let is_exported = is_public(node, content);
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Constant,
                    line: node.start_position().row + 1,
                });
                if is_exported {
                    exports.push(name);
                }
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            extract_from_node(&child, content, imports, exports, definitions);
        }
    }
}

fn extract_use(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "use_clause"
                || child.kind() == "scoped_use_list"
                || child.kind() == "use_list"
            {
                extract_use_path(&child, content, imports);
            }
        }
    }
}

fn extract_use_path(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    let kind = node.kind();

    if kind == "use_list" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_use_path(&child, content, imports);
            }
        }
        return;
    }

    if kind == "scoped_identifier" || kind == "identifier" || kind == "scoped_use_list" {
        if let Ok(path) = node.utf8_text(content.as_bytes()) {
            let cleaned = path
                .replace("crate::", "")
                .replace("super::", "")
                .replace("self::", "");
            if !cleaned.is_empty() && !cleaned.starts_with('{') {
                imports.push(cleaned);
            }
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_use_path(&child, content, imports);
            }
        }
    }
}

fn extract_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn is_public(node: &tree_sitter::Node, content: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                if let Ok(text) = child.utf8_text(content.as_bytes()) {
                    return text.contains("pub");
                }
            }
        }
    }

    if let Some(parent) = node.parent() {
        for i in 0..parent.child_count() {
            if let Some(sibling) = parent.child(i) {
                if sibling.id() == node.id() && i > 0 {
                    if let Some(prev) = parent.child(i - 1) {
                        if prev.kind() == "visibility_modifier" {
                            if let Ok(text) = prev.utf8_text(content.as_bytes()) {
                                return text.contains("pub");
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_imports() {
        let code = r#"
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::User;
"#;
        let result = parse(Path::new("lib.rs"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("serde")));
    }

    #[test]
    fn test_parse_rust_struct() {
        let code = r#"
pub struct User {
    pub id: String,
    pub name: String,
}

impl User {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}
"#;
        let result = parse(Path::new("user.rs"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "User"));
        assert!(result.exports.contains(&"User".to_string()));
    }
}

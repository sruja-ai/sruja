//! Python parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
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
        "import_statement" => {
            extract_import(node, content, imports);
        }
        "import_from_statement" => {
            extract_from_import(node, content, imports);
        }
        "function_definition" => {
            if let Some(name) = extract_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
                if name.starts_with('_') {
                    exports.push(name);
                }
            }
        }
        "class_definition" => {
            if let Some(name) = extract_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
                if !name.starts_with('_') {
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

fn extract_import(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "dotted_name" || child.kind() == "aliased_import" {
                if let Ok(name) = child.utf8_text(content.as_bytes()) {
                    let module = name.split_whitespace().last().unwrap_or(name);
                    imports.push(module.to_string());
                }
            }
        }
    }
}

fn extract_from_import(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    let mut found_from = false;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "from" {
                found_from = true;
            } else if child.kind() == "dotted_name" && found_from {
                if let Ok(name) = child.utf8_text(content.as_bytes()) {
                    imports.push(name.to_string());
                    break;
                }
            }
        }
    }
}

fn extract_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_python_imports() {
        let code = r#"
import os
import sys
from flask import Flask, request
from .models import User
"#;
        let result = parse(Path::new("app.py"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("flask")));
    }

    #[test]
    fn test_parse_python_classes() {
        let code = r#"
class UserService:
    def __init__(self, db):
        self.db = db
    
    def get_user(self, id):
        return self.db.find(id)
"#;
        let result = parse(Path::new("service.py"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
        assert!(result.exports.contains(&"UserService".to_string()));
    }
}

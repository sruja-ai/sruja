//! C parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).ok()?;

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
        "preproc_include" => {
            extract_include(node, content, imports);
        }
        "function_definition" => {
            if let Some(name) = extract_function_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "struct_specifier" => {
            if let Some(name) = extract_struct_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Struct,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "enum_specifier" => {
            if let Some(name) = extract_enum_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "type_definition" => {
            if let Some(name) = extract_typedef_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Struct,
                    line: node.start_position().row + 1,
                });
            }
        }
        "declaration" => {
            if let Some((name, is_const)) = extract_global_var(node, content) {
                if is_const {
                    definitions.push(Definition {
                        name,
                        kind: DefinitionKind::Constant,
                        line: node.start_position().row + 1,
                    });
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

fn extract_include(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                if let Ok(text) = child.utf8_text(content.as_bytes()) {
                    let path = text.trim_matches('"').trim_matches('<').trim_matches('>');
                    imports.push(path.to_string());
                }
            }
        }
    }
}

fn extract_function_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "function_declarator" {
                for j in 0..child.child_count() {
                    if let Some(declarator) = child.child(j) {
                        if declarator.kind() == "identifier" {
                            return declarator
                                .utf8_text(content.as_bytes())
                                .ok()
                                .map(|s| s.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn extract_struct_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_enum_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_typedef_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    let mut last_identifier = None;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_identifier" {
                last_identifier = child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    last_identifier
}

fn extract_global_var(node: &tree_sitter::Node, content: &str) -> Option<(String, bool)> {
    let mut _is_const = false;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "type_qualifier" {
                if let Ok(text) = child.utf8_text(content.as_bytes()) {
                    if text == "const" {
                        let _is_public = true;
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_c_includes() {
        let code = r#"
#include <stdio.h>
#include "myheader.h"
"#;
        let result = parse(Path::new("main.c"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("stdio")));
    }

    #[test]
    fn test_parse_c_function() {
        let code = r#"
int add(int a, int b) {
    return a + b;
}

struct User {
    char* name;
    int age;
};
"#;
        let result = parse(Path::new("utils.c"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "add"));
    }
}

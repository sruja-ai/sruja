//! C++ parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_cpp::LANGUAGE.into())
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
        "preproc_include" => {
            extract_include(node, content, imports);
        }
        "namespace_definition" => {
            if let Some(name) = extract_namespace_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
            }
        }
        "class_specifier" => {
            if let Some(name) = extract_class_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Class,
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
        "declaration" => {
            if let Some((name, is_const)) = extract_const_var(node, content) {
                if is_const {
                    definitions.push(Definition {
                        name,
                        kind: DefinitionKind::Constant,
                        line: node.start_position().row + 1,
                    });
                }
            }
        }
        "template_declaration" => {
            extract_template_definitions(node, content, definitions, exports);
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

fn extract_namespace_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" || child.kind() == "nested_namespace_specifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_class_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
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

fn extract_function_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "function_declarator" {
                for j in 0..child.child_count() {
                    if let Some(declarator) = child.child(j) {
                        if declarator.kind() == "identifier"
                            || declarator.kind() == "qualified_identifier"
                            || declarator.kind() == "destructor_name"
                        {
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

fn extract_const_var(node: &tree_sitter::Node, content: &str) -> Option<(String, bool)> {
    let mut is_const = false;
    let mut name = None;

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "type_qualifier" => {
                    if let Ok(text) = child.utf8_text(content.as_bytes()) {
                        if text == "const" || text == "constexpr" {
                            is_const = true;
                        }
                    }
                }
                "init_declarator" => {
                    for j in 0..child.child_count() {
                        if let Some(id) = child.child(j) {
                            if id.kind() == "identifier" {
                                name = id.utf8_text(content.as_bytes()).ok().map(|s| s.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    name.map(|n| (n, is_const))
}

fn extract_template_definitions(
    node: &tree_sitter::Node,
    content: &str,
    definitions: &mut Vec<Definition>,
    exports: &mut Vec<String>,
) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "function_definition" {
                if let Some(name) = extract_function_name(&child, content) {
                    definitions.push(Definition {
                        name: name.clone(),
                        kind: DefinitionKind::Function,
                        line: child.start_position().row + 1,
                    });
                    exports.push(name);
                }
            } else if child.kind() == "class_specifier" {
                if let Some(name) = extract_class_name(&child, content) {
                    definitions.push(Definition {
                        name: name.clone(),
                        kind: DefinitionKind::Class,
                        line: child.start_position().row + 1,
                    });
                    exports.push(name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpp_includes() {
        let code = r#"
#include <iostream>
#include "myclass.h"
"#;
        let result = parse(Path::new("main.cpp"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("iostream")));
    }

    #[test]
    fn test_parse_cpp_class() {
        let code = r#"
class UserService {
public:
    User findById(const std::string& id);
    
private:
    UserRepository* repository;
};
"#;
        let result = parse(Path::new("UserService.cpp"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
    }
}

//! Kotlin parser using Tree-sitter.

#![allow(dead_code)]

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    // tree-sitter-kotlin 0.3 depends on tree-sitter 0.20; we use 0.24, so we cannot
    // use the crate's Language with our Parser. Use a minimal line-based extraction instead.
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut imports = Vec::new();
    let mut definitions = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("import ") {
            let path_str = stripped.trim().trim_end_matches(';').trim();
            if !path_str.is_empty() {
                imports.push(path_str.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("class ") {
            let rest = rest.trim();
            let class_name = rest
                .split(|c: char| c.is_whitespace() || c == '(' || c == ':')
                .next()
                .unwrap_or("")
                .trim();
            if !class_name.is_empty() && class_name.chars().next().is_some_and(|c| c.is_uppercase())
            {
                definitions.push(Definition {
                    name: class_name.to_string(),
                    kind: DefinitionKind::Class,
                    line: 0,
                });
            }
        }
    }

    Some(ParsedFile {
        name,
        path: path.to_string_lossy().to_string(),
        imports,
        exports: definitions.iter().map(|d| d.name.clone()).collect(),
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
        "import_header" => {
            extract_import(node, content, imports);
        }
        "class_declaration" | "object_declaration" | "data_class_declaration" => {
            if let Some(name) = extract_type_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
                if !has_modifier(node, content, "private") {
                    exports.push(name);
                }
            }
        }
        "interface_declaration" => {
            if let Some(name) = extract_type_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Interface,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "enum_declaration" => {
            if let Some(name) = extract_type_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "function_declaration" => {
            if let Some(name) = extract_function_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "property_declaration" => {
            if let Some(name) = extract_property_name(node, content) {
                let is_const = has_modifier(node, content, "const");
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
        if let Some(child) = node.child(i as u32) {
            extract_from_node(&child, content, imports, exports, definitions);
        }
    }
}

fn extract_import(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "identifier" || child.kind() == "package_header" {
                if let Ok(path) = child.utf8_text(content.as_bytes()) {
                    imports.push(path.to_string());
                }
            }
            extract_import(&child, content, imports);
        }
    }
}

fn extract_type_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
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
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "simple_identifier" || child.kind() == "identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_property_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "variable_declaration" {
                for j in 0..child.child_count() {
                    if let Some(var_child) = child.child(j as u32) {
                        if var_child.kind() == "simple_identifier" {
                            return var_child
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

fn has_modifier(node: &tree_sitter::Node, content: &str, modifier: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "modifiers" {
                for j in 0..child.child_count() {
                    if let Some(mod_child) = child.child(j as u32) {
                        if let Ok(text) = mod_child.utf8_text(content.as_bytes()) {
                            if text == modifier {
                                return true;
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
    fn test_parse_kotlin_imports() {
        let code = r#"
import kotlinx.coroutines.*
import com.example.models.User
"#;
        let result = parse(Path::new("Service.kt"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("kotlinx")));
    }

    #[test]
    fn test_parse_kotlin_class() {
        let code = r#"
class UserService(
    private val repository: UserRepository
) {
    suspend fun findById(id: String): User? {
        return repository.find(id)
    }
}
"#;
        let result = parse(Path::new("UserService.kt"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
    }
}

//! PHP parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language =
        unsafe { std::mem::transmute(tree_sitter_php::LANGUAGE_PHP) };
    parser.set_language(&language).ok()?;

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
        "namespace_use_declaration" => {
            extract_use(node, content, imports);
        }
        "class_declaration" => {
            if let Some(name) = extract_name(node, content) {
                let is_public = true;
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
                if is_public {
                    exports.push(name);
                }
            }
        }
        "interface_declaration" => {
            if let Some(name) = extract_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Interface,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "trait_declaration" => {
            if let Some(name) = extract_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "enum_declaration" => {
            if let Some(name) = extract_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "method_declaration" | "function_definition" => {
            if let Some(name) = extract_function_name(node, content) {
                let _is_public = has_visibility(node, content, "public");
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "const_declaration" => {
            if let Some(name) = extract_const_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Constant,
                    line: node.start_position().row + 1,
                });
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
            if child.kind() == "namespace_use_clause" || child.kind() == "namespace_use_clause_list"
            {
                extract_use_clause(&child, content, imports);
            }
        }
    }
}

fn extract_use_clause(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    if node.kind() == "namespace_use_clause_list" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                extract_use_clause(&child, content, imports);
            }
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "name" || child.kind() == "qualified_name" {
                if let Ok(path) = child.utf8_text(content.as_bytes()) {
                    imports.push(path.to_string());
                }
            }
        }
    }
}

fn extract_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "name" {
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
            if child.kind() == "name" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_const_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "const_element_list" {
                for j in 0..child.child_count() {
                    if let Some(const_elem) = child.child(j) {
                        for k in 0..const_elem.child_count() {
                            if let Some(name_node) = const_elem.child(k) {
                                if name_node.kind() == "name" {
                                    return name_node
                                        .utf8_text(content.as_bytes())
                                        .ok()
                                        .map(|s| s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn has_visibility(node: &tree_sitter::Node, content: &str, visibility: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "visibility_modifier" {
                if let Ok(text) = child.utf8_text(content.as_bytes()) {
                    return text == visibility;
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
    fn test_parse_php_use() {
        let code = r#"
<?php
use App\Models\User;
use Illuminate\Support\Facades\DB;
"#;
        let result = parse(Path::new("Service.php"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("App\\Models")));
    }

    #[test]
    fn test_parse_php_class() {
        let code = r#"
<?php
namespace App\Services;

class UserService
{
    public function findById(string $id): ?User
    {
        return User::find($id);
    }
}
"#;
        let result = parse(Path::new("UserService.php"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
    }
}

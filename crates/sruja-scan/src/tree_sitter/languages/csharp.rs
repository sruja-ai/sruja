//! C# parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
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
        "using_directive" => {
            extract_using(node, content, imports);
        }
        "class_declaration" | "struct_declaration" | "record_declaration" | "enum_declaration" => {
            if let Some(name) = extract_name(node, content) {
                let is_public = has_modifier(node, content, "public");
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
                let is_public = has_modifier(node, content, "public");
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Interface,
                    line: node.start_position().row + 1,
                });
                if is_public {
                    exports.push(name);
                }
            }
        }
        "method_declaration" => {
            if let Some(name) = extract_name_from_method(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "field_declaration" => {
            if let Some(name) = extract_field_name(node, content) {
                let is_const = has_modifier(node, content, "const")
                    || has_modifier(node, content, "static")
                        && has_modifier(node, content, "readonly");
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

fn extract_using(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "qualified_name" || child.kind() == "identifier" {
                if let Ok(path) = child.utf8_text(content.as_bytes()) {
                    imports.push(path.to_string());
                }
            }
            extract_using(&child, content, imports);
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

fn extract_name_from_method(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "explicit_interface_specifier" {
                continue;
            }
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

fn extract_field_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "variable_declaration" {
                for j in 0..child.child_count() {
                    if let Some(var_declarator) = child.child(j) {
                        if var_declarator.kind() == "variable_declarator" {
                            for k in 0..var_declarator.child_count() {
                                if let Some(id) = var_declarator.child(k) {
                                    if id.kind() == "identifier" {
                                        return id
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
    }
    None
}

fn has_modifier(node: &tree_sitter::Node, content: &str, modifier: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifier_list" {
                for j in 0..child.child_count() {
                    if let Some(mod_child) = child.child(j) {
                        if let Ok(text) = mod_child.utf8_text(content.as_bytes()) {
                            if text == modifier {
                                return true;
                            }
                        }
                    }
                }
            }
            if let Ok(text) = child.utf8_text(content.as_bytes()) {
                if text == modifier {
                    return true;
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
    fn test_parse_csharp_usings() {
        let code = r#"
using System.Collections.Generic;
using Microsoft.EntityFrameworkCore;
"#;
        let result = parse(Path::new("Service.cs"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("System")));
    }

    #[test]
    fn test_parse_csharp_class() {
        let code = r#"
namespace MyApp.Services;

public class UserService
{
    private readonly IUserRepository _repository;
    
    public User FindById(string id)
    {
        return _repository.Find(id);
    }
}
"#;
        let result = parse(Path::new("UserService.cs"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
        assert!(result.exports.contains(&"UserService".to_string()));
    }
}

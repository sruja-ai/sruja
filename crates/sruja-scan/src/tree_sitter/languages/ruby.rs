//! Ruby parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_ruby::LANGUAGE.into())
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
    _exports: &mut Vec<String>,
    definitions: &mut Vec<Definition>,
) {
    let kind = node.kind();

    match kind {
        "call" => {
            extract_require(node, content, imports);
        }
        "class" => {
            if let Some(name) = extract_class_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
            }
        }
        "module" => {
            if let Some(name) = extract_module_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
            }
        }
        "method" => {
            if let Some(name) = extract_method_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "singleton_method" => {
            if let Some(name) = extract_singleton_method_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "constant" => {
            if let Ok(name) = node.utf8_text(content.as_bytes()) {
                definitions.push(Definition {
                    name: name.to_string(),
                    kind: DefinitionKind::Constant,
                    line: node.start_position().row + 1,
                });
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            extract_from_node(&child, content, imports, _exports, definitions);
        }
    }
}

fn extract_require(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    let mut method_name = None;
    let mut arg = None;

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            match child.kind() {
                "identifier" => {
                    method_name = child.utf8_text(content.as_bytes()).ok();
                }
                "argument_list" => {
                    for j in 0..child.child_count() {
                        if let Some(arg_node) = child.child(j as u32) {
                            if arg_node.kind() == "string" {
                                arg = extract_string_content(&arg_node, content);
                            }
                        }
                    }
                }
                "string" => {
                    arg = extract_string_content(&child, content);
                }
                _ => {}
            }
        }
    }

    if let (Some(method), Some(path)) = (method_name, arg) {
        if method == "require" || method == "require_relative" || method == "load" {
            imports.push(path);
        }
    }
}

fn extract_string_content(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "string_content" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    node.utf8_text(content.as_bytes())
        .ok()
        .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
}

fn extract_class_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "constant" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
            if child.kind() == "scope_resolution" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_module_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "constant" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_method_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "identifier" || child.kind() == "setter" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_singleton_method_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    let mut found_target = false;
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "identifier" && !found_target {
                if let Ok(text) = child.utf8_text(content.as_bytes()) {
                    if text == "self" {
                        found_target = true;
                    }
                }
                continue;
            }
            if found_target && child.kind() == "identifier" {
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
    fn test_parse_ruby_requires() {
        let code = r#"
require 'json'
require_relative './models/user'
"#;
        let result = parse(Path::new("service.rb"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("json")));
    }

    #[test]
    fn test_parse_ruby_class() {
        let code = r#"
class UserService
  def initialize(repo)
    @repo = repo
  end
  
  def find(id)
    @repo.find(id)
  end
end
"#;
        let result = parse(Path::new("user_service.rb"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
    }
}

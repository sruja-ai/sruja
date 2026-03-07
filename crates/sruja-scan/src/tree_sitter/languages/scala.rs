//! Scala parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_scala::LANGUAGE.into())
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
        "import_declaration" => {
            extract_import(node, content, imports);
        }
        "class_definition" | "object_definition" | "trait_definition" | "case_class_definition" => {
            if let Some(name) = extract_definition_name(node, content) {
                let def_kind = if kind == "trait_definition" {
                    DefinitionKind::Interface
                } else {
                    DefinitionKind::Class
                };
                definitions.push(Definition {
                    name: name.clone(),
                    kind: def_kind,
                    line: node.start_position().row + 1,
                });
                if !has_modifier(node, content, "private") {
                    exports.push(name);
                }
            }
        }
        "enum_definition" => {
            if let Some(name) = extract_definition_name(node, content) {
                definitions.push(Definition {
                    name: name.clone(),
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
                exports.push(name);
            }
        }
        "function_definition" | "method_definition" => {
            if let Some(name) = extract_function_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "val_definition" | "given_definition" => {
            if let Some(name) = extract_val_name(node, content) {
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

fn extract_import(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "import_selectors" {
                for j in 0..child.child_count() {
                    if let Some(selector) = child.child(j) {
                        extract_import_selector(&selector, content, imports);
                    }
                }
            } else if child.kind() == "stable_identifier" || child.kind() == "identifier" {
                if let Ok(path) = child.utf8_text(content.as_bytes()) {
                    imports.push(path.to_string());
                }
            }
            extract_import(&child, content, imports);
        }
    }
}

fn extract_import_selector(node: &tree_sitter::Node, content: &str, imports: &mut Vec<String>) {
    if node.kind() == "import_selector" {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(content.as_bytes()) {
                        imports.push(name.to_string());
                    }
                }
            }
        }
    }
}

fn extract_definition_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
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

fn extract_function_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
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

fn extract_val_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
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

fn has_modifier(node: &tree_sitter::Node, content: &str, modifier: &str) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "modifiers" {
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
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scala_imports() {
        let code = r#"
import scala.concurrent.Future
import cats.effect.IO
"#;
        let result = parse(Path::new("Service.scala"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("scala")));
    }

    #[test]
    fn test_parse_scala_class() {
        let code = r#"
class UserService(repository: UserRepository) {
  def findById(id: String): Option[User] = {
    repository.find(id)
  }
}
"#;
        let result = parse(Path::new("UserService.scala"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
    }
}

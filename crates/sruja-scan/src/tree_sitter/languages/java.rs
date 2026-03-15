//! Java parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_java::LANGUAGE.into())
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
        "class_declaration" | "record_declaration" | "enum_declaration" => {
            if let Some(name) = extract_name(node, content, "identifier") {
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
            if let Some(name) = extract_name(node, content, "identifier") {
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
            if let Some(name) = extract_name(node, content, "identifier") {
                let _is_public = has_modifier(node, content, "public");
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "constant_declaration" | "field_declaration" => {
            if let Some(name) = extract_name(node, content, "identifier") {
                let is_static = has_modifier(node, content, "static");
                if is_static {
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
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                if let Ok(path) = child.utf8_text(content.as_bytes()) {
                    imports.push(path.replace("static ", ""));
                }
            }
        }
    }
}

fn extract_name(node: &tree_sitter::Node, content: &str, target_kind: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == target_kind {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
            if let Some(name) = extract_name(&child, content, target_kind) {
                return Some(name);
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
    fn test_parse_java_imports() {
        let code = r#"
import java.util.HashMap;
import com.example.models.User;
"#;
        let result = parse(Path::new("Service.java"), code).unwrap();
        assert!(result.imports.iter().any(|i| i.contains("java.util")));
    }

    #[test]
    fn test_parse_java_class() {
        let code = r#"
public class UserService {
    private UserRepository repository;
    
    public User findById(String id) {
        return repository.find(id);
    }
}
"#;
        let result = parse(Path::new("UserService.java"), code).unwrap();
        assert!(result.definitions.iter().any(|d| d.name == "UserService"));
        assert!(result.exports.contains(&"UserService".to_string()));
    }
}

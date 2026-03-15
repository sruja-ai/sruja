//! TypeScript and JavaScript parser using Tree-sitter.

use std::path::Path;

use super::{Definition, DefinitionKind, ParsedFile};
use crate::tree_sitter::detector::Language;

pub fn parse(path: &Path, content: &str) -> Option<ParsedFile> {
    let language = if path.extension()?.to_str()? == "ts" || path.extension()?.to_str()? == "tsx" {
        Language::TypeScript
    } else {
        Language::JavaScript
    };

    let mut parser = tree_sitter::Parser::new();

    match language {
        Language::TypeScript => {
            parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .ok()?;
        }
        Language::JavaScript => {
            parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .ok()?;
        }
        _ => return None,
    }

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
            if let Some(source) = extract_import_source(node, content) {
                imports.push(source);
            }
        }
        "export_statement" => {
            extract_exports(node, content, exports, definitions);
        }
        "function_declaration" | "method_definition" | "arrow_function" => {
            if let Some(name) = extract_function_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Function,
                    line: node.start_position().row + 1,
                });
            }
        }
        "class_declaration" => {
            if let Some(name) = extract_class_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Class,
                    line: node.start_position().row + 1,
                });
            }
        }
        "interface_declaration" => {
            if let Some(name) = extract_interface_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Interface,
                    line: node.start_position().row + 1,
                });
            }
        }
        "enum_declaration" => {
            if let Some(name) = extract_enum_name(node, content) {
                definitions.push(Definition {
                    name,
                    kind: DefinitionKind::Enum,
                    line: node.start_position().row + 1,
                });
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            extract_variable_declarations(node, content, definitions);
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            extract_from_node(&child, content, imports, exports, definitions);
        }
    }
}

fn extract_import_source(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "string" || child.kind() == "string_fragment" {
                let text = child.utf8_text(content.as_bytes()).ok()?;
                let cleaned = text.trim_matches('"').trim_matches('\'').to_string();
                return Some(cleaned);
            }
            if child.kind() == "source" {
                return extract_import_source(&child, content);
            }
        }
    }
    None
}

fn extract_exports(
    node: &tree_sitter::Node,
    content: &str,
    exports: &mut Vec<String>,
    definitions: &mut Vec<Definition>,
) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            match child.kind() {
                "function_declaration" => {
                    if let Some(name) = extract_function_name(&child, content) {
                        exports.push(name.clone());
                        definitions.push(Definition {
                            name,
                            kind: DefinitionKind::Function,
                            line: child.start_position().row + 1,
                        });
                    }
                }
                "class_declaration" => {
                    if let Some(name) = extract_class_name(&child, content) {
                        exports.push(name.clone());
                        definitions.push(Definition {
                            name,
                            kind: DefinitionKind::Class,
                            line: child.start_position().row + 1,
                        });
                    }
                }
                "interface_declaration" => {
                    if let Some(name) = extract_interface_name(&child, content) {
                        exports.push(name.clone());
                        definitions.push(Definition {
                            name,
                            kind: DefinitionKind::Interface,
                            line: child.start_position().row + 1,
                        });
                    }
                }
                "identifier" => {
                    if let Ok(name) = child.utf8_text(content.as_bytes()) {
                        exports.push(name.to_string());
                    }
                }
                "export_clause" | "named_exports" => {
                    extract_exports(&child, content, exports, definitions);
                }
                _ => {}
            }
        }
    }
}

fn extract_function_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "identifier" || child.kind() == "property_identifier" {
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
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "type_identifier" || child.kind() == "identifier" {
                return child
                    .utf8_text(content.as_bytes())
                    .ok()
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

fn extract_interface_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
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

fn extract_enum_name(node: &tree_sitter::Node, content: &str) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
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

fn extract_variable_declarations(
    node: &tree_sitter::Node,
    content: &str,
    definitions: &mut Vec<Definition>,
) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i as u32) {
            if child.kind() == "variable_declarator" {
                for j in 0..child.child_count() {
                    if let Some(ident) = child.child(j as u32) {
                        if ident.kind() == "identifier" {
                            if let Ok(name) = ident.utf8_text(content.as_bytes()) {
                                definitions.push(Definition {
                                    name: name.to_string(),
                                    kind: DefinitionKind::Constant,
                                    line: node.start_position().row + 1,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typescript_imports() {
        let code = r#"
import { db } from './database';
import { User } from './models';
import * as utils from './utils';

export class UserService {
    getUser(id: string) {
        return db.find(id);
    }
}
"#;
        let result = parse(Path::new("service.ts"), code).unwrap();
        assert_eq!(result.imports.len(), 3);
        assert!(result.imports.contains(&"./database".to_string()));
    }

    #[test]
    fn test_parse_typescript_exports() {
        let code = r#"
export interface User {
    id: string;
    name: string;
}

export function getUser(id: string): User {
    return { id, name: "test" };
}
"#;
        let result = parse(Path::new("user.ts"), code).unwrap();
        assert!(result.exports.contains(&"User".to_string()));
        assert!(result.exports.contains(&"getUser".to_string()));
    }
}

use crate::workspace::Document;
use sruja_language::ast::*;
use tower_lsp::lsp_types::*;

/// Find word boundaries at a position in a line
pub fn word_bounds(line: &str, pos: usize) -> (usize, usize) {
    let pos = pos.min(line.len());
    let mut start = pos;
    while start > 0 && is_ident_char(line.chars().nth(start - 1).unwrap_or(' ')) {
        start -= 1;
    }
    let mut end = pos;
    while end < line.len() && is_ident_char(line.chars().nth(end).unwrap_or(' ')) {
        end += 1;
    }
    (start, end)
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// Collect elements from a program for quick lookup
fn collect_elements(
    program: &Program,
) -> (std::collections::HashMap<String, ElementDef>, Vec<Relation>) {
    let mut elements = std::collections::HashMap::new();
    let mut relations = Vec::new();

    for item in &program.items {
        match item {
            TopLevelItem::ElementDef(elem) => {
                elements.insert(elem.assignment.name.clone(), elem.clone());
            }
            TopLevelItem::Relation(rel) => {
                relations.push(rel.clone());
            }
            _ => {}
        }
    }

    (elements, relations)
}

/// Get the last token before the cursor position
pub fn last_token(s: &str) -> String {
    let mut i = s.len();
    while i > 0 && !is_ident_char(s.chars().nth(i - 1).unwrap_or(' ')) {
        i -= 1;
    }
    if i == 0 {
        return String::new();
    }
    let mut j = i;
    while j > 0 && is_ident_char(s.chars().nth(j - 1).unwrap_or(' ')) {
        j -= 1;
    }
    s[j..i].trim().to_string()
}

/// Find element information for hover
pub fn find_element_hover(program: &Program, id: &str) -> Option<(String, String)> {
    let (elements, _) = collect_elements(program);

    // Try exact match first
    if let Some(elem) = elements.get(id) {
        let kind = elem.assignment.kind.to_string();
        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());
        return Some((kind, title));
    }

    // Try partial match (short name)
    for (fqn, elem) in &elements {
        if fqn.ends_with(&format!(".{}", id)) || fqn == id {
            let kind = elem.assignment.kind.to_string();
            let title = elem
                .assignment
                .title
                .clone()
                .unwrap_or_else(|| elem.assignment.name.clone());
            return Some((kind, title));
        }
    }

    None
}

/// Find relation information for hover
pub fn find_relation_hover(program: &Program, from: &str, to: &str) -> Option<(String, String)> {
    let (_elements, relations) = collect_elements(program);

    for rel in relations {
        let rel_from = rel.from.as_string();
        let rel_to = rel.to.as_string();

        if (rel_from == from || rel_from.ends_with(&format!(".{}", from)))
            && (rel_to == to || rel_to.ends_with(&format!(".{}", to)))
        {
            let verb = String::new(); // TODO: Extract verb from relation
            let label = rel.label.clone().unwrap_or_default();
            return Some((verb, label));
        }
    }

    None
}

/// Get hover information at a position
pub fn get_hover(
    doc: &Document,
    program: &Program,
    line: usize,
    character: usize,
) -> Option<Hover> {
    let line_text = doc.get_line(line)?;
    if character > line_text.len() {
        return None;
    }

    let (start, end) = word_bounds(&line_text, character);
    let word: &str = line_text[start..end].trim();

    // Check if hovering over an element
    if !word.is_empty() {
        if let Some((kind, title)) = find_element_hover(program, word) {
            let content = format!("**{}** `{}`\n{}", kind, word, title);
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(Range {
                    start: Position {
                        line: line as u32,
                        character: start as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: end as u32,
                    },
                }),
            });
        }
    }

    // Check if hovering over an arrow (relation)
    if let Some(arrow_idx) = line_text.find("->") {
        if character >= arrow_idx && character < arrow_idx + 2 {
            let (left_start, left_end) = word_bounds(&line_text, arrow_idx);
            let left = line_text[left_start..left_end].trim();

            let mut right_pos = arrow_idx + 2;
            while right_pos < line_text.len()
                && !is_ident_char(line_text.chars().nth(right_pos).unwrap_or(' '))
            {
                right_pos += 1;
            }
            let (right_start, right_end) = word_bounds(&line_text, right_pos);
            let right = line_text[right_start..right_end].trim();

            if !left.is_empty() && !right.is_empty() {
                if let Some((verb, label)) = find_relation_hover(program, left, right) {
                    let mut parts = Vec::new();
                    if !verb.is_empty() {
                        parts.push(verb);
                    }
                    if !label.is_empty() {
                        parts.push(label);
                    }
                    let content =
                        format!("**Relation** `{} -> {}`\n{}", left, right, parts.join(" "));
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(Range {
                            start: Position {
                                line: line as u32,
                                character: arrow_idx as u32,
                            },
                            end: Position {
                                line: line as u32,
                                character: (arrow_idx + 2) as u32,
                            },
                        }),
                    });
                }
            }
        }
    }

    None
}

/// Get completion items at a position
pub fn get_completion(
    doc: &Document,
    program: &Program,
    line: usize,
    character: usize,
) -> Vec<CompletionItem> {
    let line_text = match doc.get_line(line) {
        Some(l) => l,
        None => return Vec::new(),
    };

    if character > line_text.len() {
        return Vec::new();
    }

    let before = &line_text[..character];
    let token: String = last_token(before);

    let mut items = Vec::new();

    // Keywords
    let keywords = vec![
        "system",
        "container",
        "component",
        "datastore",
        "database",
        "queue",
        "person",
        "adr",
        "requirement",
        "policy",
        "scenario",
        "story",
        "flow",
        "description",
        "technology",
        "tags",
        "metadata",
        "relation",
    ];

    for keyword in keywords {
        if token.is_empty() || keyword.to_lowercase().starts_with(&token.to_lowercase()) {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }
    }

    // Element IDs
    let (elements, _) = collect_elements(program);
    let mut seen = std::collections::HashSet::new();

    for (fqn, _) in &elements {
        // Add short name
        if let Some(short) = fqn.split('.').last() {
            if !seen.contains(short) {
                seen.insert(short.to_string());
                if token.is_empty() || short.to_lowercase().starts_with(&token.to_lowercase()) {
                    items.push(CompletionItem {
                        label: short.to_string(),
                        kind: Some(CompletionItemKind::TEXT),
                        ..Default::default()
                    });
                }
            }
        }

        // Add FQN
        if !seen.contains(fqn) {
            seen.insert(fqn.clone());
            if token.is_empty() || fqn.to_lowercase().starts_with(&token.to_lowercase()) {
                items.push(CompletionItem {
                    label: fqn.clone(),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
            }
        }
    }

    // Suggest verbs after arrow
    if before.contains("->") {
        let verbs = vec!["reads", "writes", "calls", "uses", "publishes"];
        for verb in verbs {
            items.push(CompletionItem {
                label: verb.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            });
        }
    }

    items
}

/// Find definition location for an identifier
pub fn find_definition(doc: &Document, program: &Program, id: &str) -> Option<Location> {
    let (elements, _) = collect_elements(program);

    // Try to find the element
    if elements.contains_key(id) {
        // Find the line where this element is declared
        for (line_idx, line) in doc.lines().iter().enumerate() {
            let trimmed = line.trim();
            let keywords = vec![
                "system",
                "container",
                "component",
                "datastore",
                "queue",
                "person",
            ];
            for keyword in keywords {
                if trimmed.starts_with(keyword) {
                    let rest = trimmed[keyword.len()..].trim();
                    if rest.starts_with(id) {
                        if let Some(col) = line.find(id) {
                            return Some(Location {
                                uri: doc.uri().clone(),
                                range: Range {
                                    start: Position {
                                        line: line_idx as u32,
                                        character: col as u32,
                                    },
                                    end: Position {
                                        line: line_idx as u32,
                                        character: (col + id.len()) as u32,
                                    },
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find all references to an identifier
pub fn find_references(doc: &Document, _program: &Program, id: &str) -> Vec<Location> {
    let mut locations = Vec::new();

    // Search in document text
    for (line_idx, line) in doc.lines().iter().enumerate() {
        let mut search_pos = 0;
        while let Some(pos) = line[search_pos..].find(id) {
            let abs_pos = search_pos + pos;
            // Check if it's a word boundary match
            let (start, end) = word_bounds(line, abs_pos);
            if line[start..end] == *id {
                locations.push(Location {
                    uri: doc.uri().clone(),
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: start as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: end as u32,
                        },
                    },
                });
            }
            search_pos = abs_pos + 1;
        }
    }

    locations
}

/// Get document symbols
pub fn get_document_symbols(doc: &Document, program: &Program) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let (elements, _) = collect_elements(program);

    for (fqn, elem) in &elements {
        let kind = match elem.assignment.kind {
            ElementKind::System => SymbolKind::CLASS,
            ElementKind::Container => SymbolKind::MODULE,
            ElementKind::Component => SymbolKind::FUNCTION,
            ElementKind::Person => SymbolKind::OBJECT,
            ElementKind::Database | ElementKind::DataStore => SymbolKind::STRUCT,
            ElementKind::Queue => SymbolKind::OBJECT,
            _ => SymbolKind::OBJECT,
        };

        let title = elem
            .assignment
            .title
            .clone()
            .unwrap_or_else(|| elem.assignment.name.clone());

        // Find line number
        let mut line = 0;
        for (line_idx, line_text) in doc.lines().iter().enumerate() {
            if line_text.contains(&elem.assignment.name) {
                line = line_idx;
                break;
            }
        }

        #[allow(deprecated)]
        symbols.push(DocumentSymbol {
            name: fqn.clone(),
            detail: Some(title),
            kind,
            range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            selection_range: Range {
                start: Position {
                    line: line as u32,
                    character: 0,
                },
                end: Position {
                    line: line as u32,
                    character: 0,
                },
            },
            children: None,
            tags: None,
            deprecated: None,
        });
    }

    symbols
}

/// Format document using DSL printer
pub fn format_document(doc: &Document, program: &Program) -> Option<Vec<TextEdit>> {
    use sruja_export::dsl::DslPrinter;

    let printer = DslPrinter::new();
    let formatted = printer.print(program);

    let text_edits: Vec<TextEdit> = vec![TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: doc.lines().len() as u32,
                character: 0,
            },
        },
        new_text: formatted,
    }];
    Some(text_edits)
}

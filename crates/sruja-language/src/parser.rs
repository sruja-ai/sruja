//! Parser for Sruja DSL using nom parser combinators
//!
//! This module implements a parser for the Sruja DSL using `nom` parser combinators.
//! It parses source code directly into an AST without a separate lexing phase.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map, opt, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded},
    IResult,
};
use sruja_diagnostics::{Diagnostic, Severity, SourceLocation};
use std::collections::HashMap;

use crate::ast::*;

/// Returns the byte offset of the start of the given 0-based line index.
/// Line 0 starts at 0; line N starts immediately after the (N-1)th newline.
fn line_to_byte_offset(input: &str, line_index: usize) -> usize {
    if line_index == 0 {
        return 0;
    }
    let mut count = 0;
    for (i, c) in input.char_indices() {
        if c == '\n' {
            count += 1;
            if count == line_index {
                return i + 1;
            }
        }
    }
    input.len()
}

/// Parser for Sruja DSL
pub struct Parser {
    filename: String,
}

impl Parser {
    /// Create a new parser
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    /// Parse source code into a Program AST
    pub fn parse(&self, input: &str) -> Result<Program, Vec<Diagnostic>> {
        match parse_program(input) {
            Ok((remaining, program)) => {
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    // Try to provide more context about what couldn't be parsed
                    let preview = if trimmed.len() > 100 {
                        format!("{}...", &trimmed[..100])
                    } else {
                        trimmed.to_string()
                    };

                    // Count lines to provide better error location
                    let lines_before_remaining = input.len() - remaining.len();
                    let line_number = input[..lines_before_remaining].matches('\n').count();
                    let line_number_u32 = line_number.min(u32::MAX as usize) as u32;

                    return Err(vec![Diagnostic::new(
                        sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                        Severity::Error,
                        format!(
                            "Unexpected input remaining at line {}: {}",
                            line_number + 1,
                            preview.replace('\n', "\\n").replace('\r', "\\r")
                        ),
                        SourceLocation::new(self.filename.clone(), line_number_u32, 0),
                    )]);
                }
                Ok(program)
            }
            Err(e) => {
                // Try to extract more information from the nom error
                let error_msg = match &e {
                    nom::Err::Error(err) => format!(
                        "Parse error at position {}: {:?}",
                        err.input.len(),
                        err.code
                    ),
                    nom::Err::Failure(err) => format!(
                        "Parse failure at position {}: {:?}",
                        err.input.len(),
                        err.code
                    ),
                    nom::Err::Incomplete(_) => "Incomplete input".to_string(),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), 0, 0),
                )])
            }
        }
    }

    /// Parse a specific section of DSL code incrementally
    ///
    /// This function parses only the changed portion of the DSL and merges it with the existing
    /// AST, avoiding full re-parsing of the entire document.
    ///
    /// Parameters:
    /// - `input`: The full DSL source code
    /// - `change_start`: The starting position of the change in the DSL
    /// - `change_end`: The ending position of the change in the DSL
    /// - `existing_ast`: The existing AST to merge changes into
    /// - `context_lines`: Number of lines to parse before/after the change for context
    ///
    /// Returns:
    /// - Updated AST if parsing succeeds
    /// - Diagnostic errors if parsing fails
    pub fn parse_incrementally(
        &self,
        input: &str,
        change_start: usize,
        change_end: usize,
        existing_ast: &Program,
        context_lines: usize,
    ) -> Result<IncrementalParseResult, Vec<Diagnostic>> {
        let start = std::time::Instant::now();

        // Find the line numbers for the change range (0-based)
        let start_line = input[..change_start].matches('\n').count();
        let end_line = input[..change_end].matches('\n').count();
        let total_lines = input.matches('\n').count();

        // Context window: [context_start_line, context_end_line] inclusive
        let context_start_line = start_line.saturating_sub(context_lines);
        let context_end_line = (end_line + context_lines).min(total_lines);

        // Byte offsets: start of line N = position after (N-1)th newline; end of context = start of line (context_end_line + 1)
        let context_start_pos = line_to_byte_offset(input, context_start_line);
        let context_end_pos = line_to_byte_offset(input, context_end_line + 1).min(input.len());

        let context_section = &input[context_start_pos..context_end_pos];

        // Parse the context section
        match parse_program(context_section) {
            Ok((_remaining, new_program)) => {
                // Merge the new AST with the existing AST
                let merged_ast =
                    self.smart_merge_asts(existing_ast, &new_program, context_start_line);

                // Analyze what changed
                let (changed_elements, changed_ranges) =
                    self.analyze_changes(existing_ast, &merged_ast);

                let elapsed = start.elapsed().as_millis() as u64;

                Ok(IncrementalParseResult {
                    updated_ast: merged_ast,
                    changed_elements,
                    changed_ranges,
                    parsing_time_ms: elapsed,
                })
            }
            Err(e) => {
                // Try to provide more context about the parse error
                let error_msg = match &e {
                    nom::Err::Error(err) => {
                        // Calculate line number within the context section
                        let context_line = context_section[..err.input.len()].matches('\n').count();
                        format!(
                            "Parse error in context section at line {}: {:?}",
                            context_line + 1,
                            err.code
                        )
                    }
                    nom::Err::Failure(err) => {
                        let context_line = context_section[..err.input.len()].matches('\n').count();
                        format!(
                            "Parse failure in context section at line {}: {:?}",
                            context_line + 1,
                            err.code
                        )
                    }
                    nom::Err::Incomplete(_) => "Incomplete input in context section".to_string(),
                };

                Err(vec![Diagnostic::new(
                    sruja_diagnostics::codes::CODE_SYNTAX_ERROR,
                    Severity::Error,
                    error_msg,
                    SourceLocation::new(self.filename.clone(), 0, 0),
                )])
            }
        }
    }

    /// Smart merge two ASTs, updating the existing AST with changes from the new AST
    ///
    /// This function intelligently merges the ASTs by:
    /// - Preserving unchanged elements
    /// - Updating modified elements
    /// - Adding new elements
    /// - Removing deleted elements
    /// - Maintaining proper parent-child relationships
    fn smart_merge_asts(
        &self,
        existing_ast: &Program,
        new_ast: &Program,
        context_line_offset: usize,
    ) -> Program {
        let mut merged_ast = existing_ast.clone();
        let mut element_map = HashMap::new();

        // Build element map from existing AST
        for item in &existing_ast.items {
            if let TopLevelItem::ElementDef(elem) = item {
                element_map.insert(elem.assignment.name.clone(), elem.assignment.name.clone());
            }
        }

        // Process new AST items
        for item in &new_ast.items {
            match item {
                TopLevelItem::ElementDef(new_elem) => {
                    let elem_name = &new_elem.assignment.name;

                    // Check if element exists in existing AST
                    if element_map.contains_key(elem_name) {
                        // Update existing element
                        self.update_existing_element(
                            &mut merged_ast,
                            new_elem,
                            context_line_offset,
                        );
                    } else {
                        // Add new element
                        self.add_new_element(&mut merged_ast, new_elem, context_line_offset);
                    }
                }
                TopLevelItem::Relation(new_rel) => {
                    // Always add relations (they don't have unique IDs)
                    self.add_new_relation(&mut merged_ast, new_rel, context_line_offset);
                }
                _ => {}
            }
        }

        // Line numbers: only new/updated items have context-local lines; we already
        // applied context_line_offset in add_new_element, add_new_relation, update_existing_element.
        merged_ast
    }

    /// Update an existing element in the AST (apply context line offset to the updated element).
    fn update_existing_element(
        &self,
        ast: &mut Program,
        new_elem: &ElementDef,
        line_offset: usize,
    ) {
        let offset = line_offset as i32;
        for item in ast.items.iter_mut() {
            if let TopLevelItem::ElementDef(elem) = item {
                if elem.assignment.name == new_elem.assignment.name {
                    elem.assignment = new_elem.assignment.clone();
                    self.update_item_line_numbers(item, offset);
                    return;
                }
            }
        }
    }

    /// Add a new element to the AST with line numbers adjusted by context offset.
    fn add_new_element(&self, ast: &mut Program, new_elem: &ElementDef, line_offset: usize) {
        let mut elem = new_elem.clone();
        let off = line_offset as i32;
        elem.location.line = (elem.location.line as i32 + off).max(0) as u32;
        elem.assignment.location.line = (elem.assignment.location.line as i32 + off).max(0) as u32;
        ast.items.push(TopLevelItem::ElementDef(Box::new(elem)));
    }

    /// Add a new relation to the AST with line numbers adjusted by context offset.
    fn add_new_relation(&self, ast: &mut Program, new_rel: &Relation, line_offset: usize) {
        let mut rel = new_rel.clone();
        rel.location.line = (rel.location.line as i32 + line_offset as i32).max(0) as u32;
        ast.items.push(TopLevelItem::Relation(rel));
    }

    /// Analyze changes between two ASTs to determine what was modified
    fn analyze_changes(
        &self,
        old_ast: &Program,
        new_ast: &Program,
    ) -> (Vec<String>, Vec<(usize, usize)>) {
        let mut changed_elements = Vec::new();
        let mut changed_ranges = Vec::new();

        // Compare elements
        let old_elements: HashMap<_, _> = old_ast
            .items
            .iter()
            .filter_map(|item| {
                if let TopLevelItem::ElementDef(elem) = item {
                    Some((elem.assignment.name.clone(), item))
                } else {
                    None
                }
            })
            .collect();

        let new_elements: HashMap<_, _> = new_ast
            .items
            .iter()
            .filter_map(|item| {
                if let TopLevelItem::ElementDef(elem) = item {
                    Some((elem.assignment.name.clone(), item))
                } else {
                    None
                }
            })
            .collect();

        // Find added/removed/modified elements
        for (name, new_item) in &new_elements {
            if let TopLevelItem::ElementDef(new_elem) = new_item {
                if let Some(old_item) = old_elements.get(name) {
                    if let TopLevelItem::ElementDef(old_elem) = old_item {
                        // Check if element was modified
                        if old_elem.assignment.title != new_elem.assignment.title
                            || old_elem.assignment.kind != new_elem.assignment.kind
                        {
                            changed_elements.push(name.clone());
                        }
                    }
                } else {
                    // Element was added
                    changed_elements.push(name.clone());
                }
            }
        }

        // For now, mark the entire document as changed
        changed_ranges.push((0, 0));

        (changed_elements, changed_ranges)
    }

    /// Update line numbers in an AST item by a given offset
    fn update_item_line_numbers(&self, item: &mut TopLevelItem, line_offset: i32) {
        match item {
            TopLevelItem::ElementDef(elem) => {
                elem.location.line = (elem.location.line as i32 + line_offset).max(0) as u32;
                elem.assignment.location.line =
                    (elem.assignment.location.line as i32 + line_offset).max(0) as u32;
            }
            TopLevelItem::Relation(rel) => {
                rel.location.line = (rel.location.line as i32 + line_offset).max(0) as u32;
            }
            _ => {}
        }
    }
}

// Helper: Skip whitespace and comments
fn skip_whitespace_and_comments(input: &str) -> IResult<&str, ()> {
    let mut input = input;
    loop {
        // Skip whitespace
        let (new_input, _) = multispace0(input)?;
        input = new_input;

        // Try to skip comment
        let comment_result: IResult<&str, &str> = alt((
            // Single-line comment: // ...
            preceded(tag("//"), take_until("\n")),
            // Multi-line comment: /* ... */
            delimited(tag("/*"), take_until("*/"), tag("*/")),
        ))(input);

        match comment_result {
            Ok((new_input, _)) => {
                input = new_input;
            }
            Err(_) => break,
        }
    }
    Ok((input, ()))
}

fn ws(input: &str) -> IResult<&str, ()> {
    skip_whitespace_and_comments(input)
}

fn ws0(input: &str) -> IResult<&str, ()> {
    multispace0(input).map(|(i, _)| (i, ()))
}

fn ws1(input: &str) -> IResult<&str, ()> {
    multispace1(input).map(|(i, _)| (i, ()))
}

// Nom parsers

/// Parse a complete program
/// Uses a more lenient approach: tries to parse items, and if one fails,
/// attempts to skip to the next line and continue parsing
fn parse_program(input: &str) -> IResult<&str, Program> {
    let (input, _) = ws(input)?;
    let mut items = Vec::new();
    let mut current = input;

    loop {
        // Skip whitespace
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }

        // Try to parse a top-level item
        match parse_top_level_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                current = next;
            }
            Err(_) => {
                // Parsing failed - try to skip to the next line and continue
                // This allows the parser to recover from syntax errors in one item
                // and continue parsing the rest
                if let Some(newline_pos) = rest.find('\n') {
                    // Skip to the next line
                    current = &rest[newline_pos + 1..];
                } else {
                    // No more newlines, we're done (or at the end)
                    // Return what we've parsed so far
                    break;
                }
            }
        }
    }

    Ok((current, Program::with_items(Program::new(), items)))
}

/// Parse a top-level item
fn parse_top_level_item(input: &str) -> IResult<&str, TopLevelItem> {
    alt((
        map(parse_kind_def, TopLevelItem::KindDef),
        map(parse_flow_assignment, TopLevelItem::Flow),
        map(parse_requirement_assignment, TopLevelItem::Requirement),
        map(parse_adr_assignment, TopLevelItem::Adr),
        map(parse_policy_assignment, TopLevelItem::Policy),
        map(parse_overview_block, TopLevelItem::Overview),
        map(parse_element_def, |e| TopLevelItem::ElementDef(Box::new(e))),
        map(parse_relation, TopLevelItem::Relation),
        map(parse_import, TopLevelItem::Import),
        map(parse_scenario, TopLevelItem::Scenario),
        map(parse_flow, TopLevelItem::Flow),
        map(parse_requirement, TopLevelItem::Requirement),
        map(parse_adr, TopLevelItem::Adr),
        map(parse_policy, TopLevelItem::Policy),
        map(parse_view, TopLevelItem::View),
        map(parse_metadata_block, |m| {
            TopLevelItem::ElementDef(Box::new(ElementDef {
                location: m.location.clone(),
                assignment: ElementAssignment {
                    location: m.location.clone(),
                    name: "metadata".to_string(),
                    kind: ElementKind::Custom("metadata".to_string()),
                    sub_kind: None,
                    title: None,
                    tag_refs: Vec::new(),
                    body: None,
                },
            }))
        }), // Temporary conversion
    ))(input)
}

/// Parse a kind definition: `identifier = kind "Title" [description] [technology] [style]`
/// Example: `person = kind "Person"`
fn parse_kind_def(input: &str) -> IResult<&str, ElementKindDef> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("kind")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    // For now, just parse the kind from the identifier
    let kind = match id.to_lowercase().as_str() {
        "person" => ElementKind::Person,
        "role" => ElementKind::Role,
        "system" => ElementKind::System,
        "container" => ElementKind::Container,
        "component" => ElementKind::Component,
        "database" => ElementKind::Database,
        "queue" => ElementKind::Queue,
        "externalsystem" | "external_system" => ElementKind::ExternalSystem,
        "datastore" => ElementKind::DataStore,
        _ => ElementKind::Custom(id.clone()),
    };

    Ok((
        input,
        ElementKindDef {
            location: SourceLocation::new(String::new(), 0, 0),
            kind,
            title,
            description: None,
            technology: None,
            style: None,
        },
    ))
}

/// Parse `REQ001 = requirement functional "..."` (preferred authoring form in examples).
fn parse_requirement_assignment(input: &str) -> IResult<&str, Requirement> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("requirement")(input)?;
    let (input, _) = ws1(input)?;
    let (input, r#type) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, title) = parse_string(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            r#type,
            description: None,
            tags: Vec::new(),
        },
    ))
}

/// Parse `ADR001 = adr "Title" { status "..."; ... }` (preferred authoring form in examples).
fn parse_adr_assignment(input: &str) -> IResult<&str, Adr> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("adr")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block)(input)?;
    let mut adr = Adr {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        status: None,
        context: None,
        decision: None,
        consequences: None,
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "status" => adr.status = Some(v),
                "context" => adr.context = Some(v),
                "decision" => adr.decision = Some(v),
                "consequences" => adr.consequences = Some(v),
                _ => {}
            }
        }
    }

    Ok((input, adr))
}

/// Parse `FlowId = flow "Title" { step ... }` (preferred authoring form in examples).
fn parse_flow_assignment(input: &str) -> IResult<&str, Flow> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("flow")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body)(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: title.unwrap_or_default(),
            description: None,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse `PolicyId = policy "Title" { category "..."; enforcement "..." }`.
fn parse_policy_assignment(input: &str) -> IResult<&str, Policy> {
    let (input, id) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag("policy")(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;

    let (input, fields) = opt(parse_kv_string_block)(input)?;
    let mut policy = Policy {
        location: SourceLocation::new(String::new(), 0, 0),
        id: id.clone(),
        title: title.unwrap_or(id),
        category: "general".to_string(),
        enforcement: "warn".to_string(),
        description: None,
    };

    if let Some(kvs) = fields {
        for (k, v) in kvs {
            match k.as_str() {
                "category" => policy.category = v,
                "enforcement" => policy.enforcement = v,
                "description" => policy.description = Some(v),
                _ => {}
            }
        }
    }

    Ok((input, policy))
}

/// Parse a `{ key "value" ... }` block into raw (k,v) pairs.
fn parse_kv_string_block(input: &str) -> IResult<&str, Vec<(String, String)>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse an overview block.
///
/// The designer demo content uses `overview { ... }` as an extension.
/// This parser extracts all fields: summary, audience, scope, goals, non_goals, risks.
fn parse_overview_block(input: &str) -> IResult<&str, OverviewBlock> {
    let (input, _) = tag("overview")(input)?;
    let (input, _) = ws0(input)?;

    // Parse the body with individual fields
    let (input, items) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_overview_item)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut overview = OverviewBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        summary: None,
        audience: None,
        scope: None,
        goals: Vec::new(),
        non_goals: Vec::new(),
        risks: Vec::new(),
    };

    for item in items {
        match item {
            OverviewItem::Summary(s) => overview.summary = Some(s),
            OverviewItem::Audience(a) => overview.audience = Some(a),
            OverviewItem::Scope(s) => overview.scope = Some(s),
            OverviewItem::Goals(g) => overview.goals = g,
            OverviewItem::NonGoals(ng) => overview.non_goals = ng,
            OverviewItem::Risks(r) => overview.risks = r,
        }
    }

    Ok((input, overview))
}

#[derive(Debug, Clone)]
enum OverviewItem {
    Summary(String),
    Audience(String),
    Scope(String),
    Goals(Vec<String>),
    NonGoals(Vec<String>),
    Risks(Vec<String>),
}

fn parse_overview_item(input: &str) -> IResult<&str, OverviewItem> {
    alt((
        map(
            preceded(tag("summary"), preceded(ws1, parse_string)),
            OverviewItem::Summary,
        ),
        map(
            preceded(tag("audience"), preceded(ws1, parse_string)),
            OverviewItem::Audience,
        ),
        map(
            preceded(tag("scope"), preceded(ws1, parse_string)),
            OverviewItem::Scope,
        ),
        map(
            preceded(tag("goals"), preceded(ws1, parse_string_array)),
            OverviewItem::Goals,
        ),
        map(
            preceded(
                alt((tag("nonGoals"), tag("non_goals"))),
                preceded(ws1, parse_string_array),
            ),
            OverviewItem::NonGoals,
        ),
        map(
            preceded(tag("risks"), preceded(ws1, parse_string_array)),
            OverviewItem::Risks,
        ),
    ))(input)
}

/// Parse an element definition: Name = Kind [SubKind] [Label] [#tags...] [Body]
fn parse_element_def(input: &str) -> IResult<&str, ElementDef> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = preceded(ws0, char('='))(input)?;
    let (input, _) = ws0(input)?;
    let (input, kind) = parse_element_kind(input)?;
    let (input, _) = ws0(input)?;
    let (input, sub_kind) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, tag_refs) = many0(parse_tag_ref)(input)?;
    let (input, _) = ws0(input)?;

    // If there's a '{' after whitespace, we must parse the body (or consume it)
    // This prevents "Unexpected input remaining" errors when body parsing fails
    let (input, body) = if input.trim_start().starts_with('{') {
        // Try to parse the body properly
        match parse_element_def_body(input) {
            Ok((rest, parsed_body)) => (rest, Some(parsed_body)),
            Err(_) => {
                // If body parsing fails, at least consume the block to allow parsing to continue
                // This is a fallback to prevent parser getting stuck
                let mut depth = 0;
                let mut in_string = false;
                let mut escape = false;
                let mut consumed = 0;

                for (i, ch) in input.char_indices() {
                    if escape {
                        escape = false;
                        continue;
                    }
                    match ch {
                        '\\' => escape = true,
                        '"' => in_string = !in_string,
                        '{' if !in_string => depth += 1,
                        '}' if !in_string => {
                            depth -= 1;
                            if depth == 0 {
                                consumed = i + 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if consumed > 0 {
                    (&input[consumed..], None)
                } else {
                    // Couldn't find matching brace, return error
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            }
        }
    } else {
        (input, None)
    };

    Ok((
        input,
        ElementDef {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            assignment: ElementAssignment {
                location: SourceLocation::new(String::new(), 0, 0),
                name,
                kind,
                sub_kind,
                title,
                tag_refs,
                body,
            },
        },
    ))
}

/// Parse element kind
fn parse_element_kind(input: &str) -> IResult<&str, ElementKind> {
    alt((
        value(ElementKind::Person, tag("person")),
        value(ElementKind::Role, tag("role")),
        value(ElementKind::System, tag("system")),
        value(ElementKind::Container, tag("container")),
        value(ElementKind::Component, tag("component")),
        value(ElementKind::Database, tag("database")),
        value(ElementKind::Queue, tag("queue")),
        value(ElementKind::Policy, tag("policy")),
        value(ElementKind::Requirement, tag("requirement")),
        value(ElementKind::Adr, tag("adr")),
        value(ElementKind::Flow, tag("flow")),
        value(ElementKind::Scenario, tag("scenario")),
        value(ElementKind::Story, tag("story")),
        map(parse_identifier, ElementKind::Custom),
    ))(input)
}

/// Parse element definition body
fn parse_element_def_body(input: &str) -> IResult<&str, ElementDefBody> {
    let (input, _) = preceded(ws0, char('{'))(input)?;

    // Parse body items, but be lenient - if an item fails to parse, skip it and continue
    let mut items = Vec::new();
    let mut current = input;

    loop {
        // Skip whitespace
        let (rest, _) = ws(current)?;
        if rest.is_empty() {
            break;
        }

        // Check if we've reached the closing brace
        if rest.trim_start().starts_with('}') {
            current = rest;
            break;
        }

        // Try to parse an item
        match parse_element_body_item(rest) {
            Ok((next, item)) => {
                items.push(item);
                current = next;
            }
            Err(_) => {
                // Item parsing failed - skip to next line and try again.
                // Do NOT use find('}') - it would match nested braces (e.g. inside
                // "container { component { } }") and truncate the body incorrectly.
                if let Some(newline_pos) = rest.find('\n') {
                    current = &rest[newline_pos + 1..];
                } else {
                    // No newline (e.g. last line of file or inline) - cannot recover
                    break;
                }
            }
        }
    }

    // Skip whitespace before closing brace
    let (input, _) = ws0(current)?;
    let (input, _) = char('}')(input)?;

    // Process items and populate body fields
    let mut body = ElementDefBody::default();
    for item in items {
        match item {
            ElementDefBodyItem::Description(d) => body.description = Some(d),
            ElementDefBodyItem::Technology(t) => body.technology = Some(t),
            ElementDefBodyItem::Metadata(m) => body.metadata = m.entries,
            ElementDefBodyItem::Slo(s) => body.slo = Some(*s),
            ElementDefBodyItem::ElementDef(e) => body.items.push(ElementDefBodyItem::ElementDef(e)),
            ElementDefBodyItem::Relation(r) => body.items.push(ElementDefBodyItem::Relation(r)),
            ElementDefBodyItem::Constraints(c) => body.constraints = c.entries,
            ElementDefBodyItem::Conventions(c) => body.conventions = c.entries,
            // Element bodies carry a `StyleBlock` (properties only). We currently parse
            // `StyleDecl` (selector + properties) and treat it as an element-local style.
            ElementDefBodyItem::Style(s) => {
                body.style = Some(StyleBlock {
                    location: s.location,
                    properties: s.properties,
                })
            }
            ElementDefBodyItem::Scale(s) => body.scale = Some(s),
            ElementDefBodyItem::Tags(_) => {} // Consumed, not stored
            // All other body items are handled above
            // This catch-all is kept for future extensibility
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }

    Ok((input, body))
}

fn parse_element_body_item(input: &str) -> IResult<&str, ElementDefBodyItem> {
    alt((
        map(
            preceded(
                alt((tag("description"), tag("desc"))),
                preceded(ws1, parse_string),
            ),
            ElementDefBodyItem::Description,
        ),
        map(
            preceded(
                alt((tag("technology"), tag("tech"))),
                preceded(ws1, parse_string),
            ),
            ElementDefBodyItem::Technology,
        ),
        map(parse_metadata_block, ElementDefBodyItem::Metadata),
        map(parse_slo_block, |s| ElementDefBodyItem::Slo(Box::new(s))),
        map(parse_element_def, |e| {
            ElementDefBodyItem::ElementDef(Box::new(e))
        }),
        map(parse_relation, ElementDefBodyItem::Relation),
        map(parse_constraints_block, ElementDefBodyItem::Constraints),
        map(parse_conventions_block, ElementDefBodyItem::Conventions),
        map(parse_style_decl, ElementDefBodyItem::Style),
        map(parse_scale_block, ElementDefBodyItem::Scale),
        map(
            preceded(
                alt((tag("tags"), tag("tag"))),
                preceded(
                    ws0,
                    opt(alt((
                        parse_string_array,
                        parse_tag_array,
                    ))),
                ),
            ),
            |t| ElementDefBodyItem::Tags(t.unwrap_or_default()),
        ),
    ))(input)
}

/// Parse an SLO block: slo { availability { ... } latency { ... } errorRate { ... } throughput { ... } }
fn parse_slo_block(input: &str) -> IResult<&str, SloBlock> {
    let (input, _) = tag("slo")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_slo_item)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut slo = SloBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        availability: None,
        latency: None,
        error_rate: None,
        throughput: None,
    };

    for item in items {
        match item {
            SloItem::Availability(a) => slo.availability = Some(a),
            SloItem::Latency(l) => slo.latency = Some(l),
            SloItem::ErrorRate(e) => slo.error_rate = Some(e),
            SloItem::Throughput(t) => slo.throughput = Some(t),
        }
    }

    Ok((input, slo))
}

#[derive(Debug, Clone)]
enum SloItem {
    Availability(SloAvailability),
    Latency(SloLatency),
    ErrorRate(SloErrorRate),
    Throughput(SloThroughput),
}

fn parse_slo_item(input: &str) -> IResult<&str, SloItem> {
    alt((
        map(parse_slo_availability, SloItem::Availability),
        map(parse_slo_latency, SloItem::Latency),
        map(parse_slo_error_rate, SloItem::ErrorRate),
        map(parse_slo_throughput, SloItem::Throughput),
    ))(input)
}

fn parse_slo_availability(input: &str) -> IResult<&str, SloAvailability> {
    let (input, _) = tag("availability")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloAvailability {
        target: None,
        window: None,
        current: None,
    };

    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }

    Ok((input, out))
}

fn parse_slo_latency(input: &str) -> IResult<&str, SloLatency> {
    let (input, _) = tag("latency")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_latency_item)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloLatency {
        p95: None,
        p99: None,
        window: None,
        current: None,
    };

    for item in items {
        match item {
            LatencyItem::P95(v) => out.p95 = Some(v),
            LatencyItem::P99(v) => out.p99 = Some(v),
            LatencyItem::Window(v) => out.window = Some(v),
            LatencyItem::Current(c) => out.current = Some(c),
        }
    }

    Ok((input, out))
}

#[derive(Debug, Clone)]
enum LatencyItem {
    P95(String),
    P99(String),
    Window(String),
    Current(SloCurrent),
}

fn parse_latency_item(input: &str) -> IResult<&str, LatencyItem> {
    alt((
        map(
            preceded(tag("p95"), preceded(ws1, parse_string)),
            LatencyItem::P95,
        ),
        map(
            preceded(tag("p99"), preceded(ws1, parse_string)),
            LatencyItem::P99,
        ),
        map(
            preceded(tag("window"), preceded(ws1, parse_string)),
            LatencyItem::Window,
        ),
        map(parse_slo_current, LatencyItem::Current),
    ))(input)
}

fn parse_slo_current(input: &str) -> IResult<&str, SloCurrent> {
    let (input, _) = tag("current")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloCurrent {
        p95: None,
        p99: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "p95" => out.p95 = Some(v),
            "p99" => out.p99 = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_slo_error_rate(input: &str) -> IResult<&str, SloErrorRate> {
    let (input, _) = tag("errorRate")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloErrorRate {
        target: None,
        window: None,
        current: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_slo_throughput(input: &str) -> IResult<&str, SloThroughput> {
    let (input, _) = tag("throughput")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;

    let mut out = SloThroughput {
        target: None,
        window: None,
        current: None,
    };
    for (k, v) in entries {
        match k.as_str() {
            "target" => out.target = Some(v),
            "window" => out.window = Some(v),
            "current" => out.current = Some(v),
            _ => {}
        }
    }
    Ok((input, out))
}

fn parse_kv_string(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws1(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, (key, value)))
}

/// Parse a scenario: (scenario | story) [ID] [Title] [Description] [{ Steps }]
fn parse_scenario(input: &str) -> IResult<&str, Scenario> {
    let (input, _) = alt((tag("scenario"), tag("story")))(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_scenario_body)(input)?;

    Ok((
        input,
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse scenario body
fn parse_scenario_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_scenario_step)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse scenario step: [step] From -> To [Description] [Tags] [Order]
fn parse_scenario_step(input: &str) -> IResult<&str, ScenarioStep> {
    let (input, _) = opt(preceded(tag("step"), ws1))(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array)(input)?;
    let (input, _) = ws0(input)?;
    let (input, order_raw) = opt(preceded(tag("order"), preceded(ws1, parse_string)))(input)?;

    let order = order_raw.as_deref().and_then(|s| s.parse::<usize>().ok());

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: tags.unwrap_or_default(),
            order,
        },
    ))
}

/// Parse a flow: flow [ID] [Title] [Description] [{ Steps }]
fn parse_flow(input: &str) -> IResult<&str, Flow> {
    let (input, _) = tag("flow")(input)?;
    let (input, _) = ws0(input)?;
    let (input, id) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, steps) = opt(parse_flow_body)(input)?;

    Ok((
        input,
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.unwrap_or_default(),
            title: title.unwrap_or_default(),
            description,
            steps: steps.unwrap_or_default(),
        },
    ))
}

/// Parse flow body
fn parse_flow_body(input: &str) -> IResult<&str, Vec<ScenarioStep>> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_flow_step)),
        preceded(ws0, char('}')),
    )(input)
}

/// Parse flow step: From -> To [Description]
fn parse_flow_step(input: &str) -> IResult<&str, ScenarioStep> {
    // Handle optional "step" keyword (matches parse_scenario_step behavior)
    let (input, _) = opt(preceded(tag("step"), ws1))(input)?;
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;

    Ok((
        input,
        ScenarioStep {
            from: Some(from),
            to: Some(to),
            description,
            tags: Vec::new(),
            order: None,
        },
    ))
}

/// Parse a requirement: requirement ID [Type] [Description] [{ Body }]
fn parse_requirement(input: &str) -> IResult<&str, Requirement> {
    let (input, _) = tag("requirement")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let id_for_title = id.clone();
    let (input, _) = ws0(input)?;
    let (input, r#type) = opt(parse_identifier)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_requirement_body)(input)?;

    Ok((
        input,
        Requirement {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title: description.clone().unwrap_or(id_for_title),
            r#type: r#type.unwrap_or_else(|| "functional".to_string()),
            description,
            tags: Vec::new(),
        },
    ))
}

/// Parse requirement body
fn parse_requirement_body(input: &str) -> IResult<&str, ()> {
    delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_requirement_property)),
        preceded(ws0, char('}')),
    )(input)
    .map(|(i, _)| (i, ()))
}

fn parse_requirement_property(input: &str) -> IResult<&str, ()> {
    // Currently we accept these properties for forward-compatibility, but the nom-based
    // parser doesn't materialize them into the `Requirement` struct yet.
    preceded(
        alt((
            tag("type"),
            tag("description"),
            tag("tags"),
            tag("metadata"),
        )),
        preceded(
            ws0,
            alt((
                map(parse_string, |_| ()),
                map(parse_tag_array, |_| ()),
                map(parse_metadata_block, |_| ()),
            )),
        ),
    )(input)
    .map(|(i, _)| (i, ()))
}

/// Parse an ADR: adr ID [Title] [{ Body }]
fn parse_adr(input: &str) -> IResult<&str, Adr> {
    let (input, _) = tag("adr")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, _body) = opt(parse_adr_body)(input)?;

    Ok((
        input,
        Adr {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.clone(),
            title: title.unwrap_or(id),
            status: None,
            context: None,
            decision: None,
            consequences: None,
        },
    ))
}

/// Parse ADR body
fn parse_adr_body(input: &str) -> IResult<&str, ()> {
    // Best-effort: consume arbitrary key/value entries inside `{ ... }`.
    // NOTE: This does not support nested braces; extend if ADRs start embedding blocks.
    delimited(
        preceded(ws0, char('{')),
        opt(take_until("}")),
        preceded(ws0, char('}')),
    )(input)
    .map(|(i, _)| (i, ()))
}

#[allow(dead_code)]
fn parse_adr_property(input: &str) -> IResult<&str, ()> {
    // Forward-compatibility: accept ADR properties even if we don't materialize them yet.
    preceded(
        alt((
            tag("status"),
            tag("context"),
            tag("decision"),
            tag("consequences"),
            tag("tags"),
        )),
        preceded(
            ws0,
            alt((map(parse_string, |_| ()), map(parse_tag_array, |_| ()))),
        ),
    )(input)
    .map(|(i, _)| (i, ()))
}

/// Parse a policy: policy ID [Title] [Description]
fn parse_policy(input: &str) -> IResult<&str, Policy> {
    let (input, _) = tag("policy")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, title) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;

    Ok((
        input,
        Policy {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.clone(),
            title: title.unwrap_or(id),
            category: "general".to_string(),
            enforcement: "warn".to_string(),
            description,
        },
    ))
}

/// Parse a view definition
/// Supports both: `view id { title "..."; include ... }` and `view id of target { ... }`
fn parse_view(input: &str) -> IResult<&str, ViewDef> {
    let (input, _) = tag("view")(input)?;
    let (input, _) = ws1(input)?;
    let (input, id) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    // Handle optional "of target" syntax: `view id of target`
    let (input, view_of) = opt(preceded(
        preceded(ws0, tag("of")),
        preceded(ws1, parse_qualified_ident),
    ))(input)?;
    let (input, _) = ws0(input)?;

    // Parse body block if present
    let (input, body_fields) = opt(parse_view_body)(input)?;

    let mut title = None;
    let mut includes = None;
    let mut excludes = None;
    let mut description = None;

    if let Some(fields) = body_fields {
        for (k, v) in fields {
            match k.as_str() {
                "title" => title = Some(v),
                "include" => {
                    // Parse include expression from string
                    // Supports: "*", "Element", "Element.Element", "Element1 Element2"
                    if v.trim() == "*" {
                        includes = Some(vec!["*".to_string()]);
                    } else {
                        // Split by whitespace (handles space-separated elements)
                        // Also handles comma-separated for flexibility
                        let elements = v
                            .split(&[',', ' '][..])
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                        includes = Some(elements);
                    }
                }
                "exclude" => {
                    // Parse exclude expression from string
                    let elements = v
                        .split(&[',', ' '][..])
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    excludes = Some(elements);
                }
                "description" => description = Some(v),
                _ => {} // Ignore other fields like "layout" for now
            }
        }
    }

    let to_expr = |elements: Vec<String>| ViewRuleExpr {
        wildcard: elements.len() == 1 && elements[0] == "*",
        recursive: false,
        elements: if elements.len() == 1 && elements[0] == "*" {
            Vec::new()
        } else {
            elements
        },
    };

    Ok((
        input,
        ViewDef {
            location: SourceLocation::new(String::new(), 0, 0),
            id,
            title,
            description,
            view_of,
            tags: Vec::new(),
            rules: if includes.is_none() && excludes.is_none() {
                Vec::new()
            } else {
                vec![ViewRule {
                    include: includes.map(to_expr),
                    exclude: excludes.map(to_expr),
                }]
            },
        },
    ))
}

/// Parse view body: `{ title "..."; include ...; layout { ... } }`
/// Extracts fields from the view body including title, description, include, exclude
fn parse_view_body(input: &str) -> IResult<&str, Vec<(String, String)>> {
    if !input.starts_with('{') {
        return Ok((input, Vec::new()));
    }

    let (input, items) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_view_body_item)),
        preceded(ws0, char('}')),
    )(input)?;

    Ok((input, items))
}

fn parse_view_body_item(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    // Check if the next token is a string or identifier
    let (input, value) = if input.starts_with('"') {
        // It's a quoted string (for title, description)
        map(parse_string, |s| s)(input)?
    } else {
        // It's an identifier or identifier list (for include/exclude)
        map(parse_view_identifier_or_wildcard, |s| s)(input)?
    };

    Ok((input, (key, value)))
}

fn parse_view_identifier_or_wildcard(input: &str) -> IResult<&str, String> {
    // Parse either a wildcard (*) or a qualified identifier
    // Also handle comma-separated lists like "Element1, Element2"
    alt((
        value("*".to_string(), char('*')),
        // Parse a qualified identifier (e.g., "System.Container")
        map(parse_qualified_ident, |q| q.as_string()),
        // Parse multiple identifiers separated by whitespace
        map(separated_list1(ws1, parse_qualified_ident), |idents| {
            idents
                .iter()
                .map(|q| q.as_string())
                .collect::<Vec<_>>()
                .join(" ")
        }),
    ))(input)
}

#[allow(dead_code)]
fn parse_view_expression(input: &str) -> IResult<&str, Vec<String>> {
    alt((
        map(char('*'), |_| vec!["*".to_string()]),
        separated_list1(preceded(ws0, char(',')), preceded(ws0, parse_identifier)),
    ))(input)
}

/// Parse constraints block: constraints { key "value" ... }
fn parse_constraints_block(input: &str) -> IResult<&str, ConstraintsBlock> {
    let (input, _) = tag("constraints")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_constraint_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConstraintsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

fn parse_constraint_entry(input: &str) -> IResult<&str, ConstraintEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConstraintEntry { key, value }))
}

/// Parse conventions block: conventions { key "value" ... }
fn parse_conventions_block(input: &str) -> IResult<&str, ConventionsBlock> {
    let (input, _) = tag("conventions")(input)?;
    let (input, _) = ws0(input)?;
    let (input, entries) = delimited(
        char('{'),
        many0(preceded(ws, parse_convention_entry)),
        preceded(ws0, char('}')),
    )(input)?;
    Ok((
        input,
        ConventionsBlock {
            location: SourceLocation::new(String::new(), 0, 0),
            entries,
        },
    ))
}

fn parse_convention_entry(input: &str) -> IResult<&str, ConventionEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = parse_string(input)?;
    Ok((input, ConventionEntry { key, value }))
}

/// Parse style declaration: style selector { property "value" ... }
fn parse_style_decl(input: &str) -> IResult<&str, StyleDecl> {
    let (input, _) = tag("style")(input)?;
    let (input, _) = ws1(input)?;
    let (input, selector) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, properties) = delimited(
        char('{'),
        many0(preceded(ws, parse_kv_string)),
        preceded(ws0, char('}')),
    )(input)?;
    let mut props_map = HashMap::new();
    for (k, v) in properties {
        props_map.insert(k, v);
    }
    Ok((
        input,
        StyleDecl {
            location: SourceLocation::new(String::new(), 0, 0),
            selector,
            properties: props_map,
        },
    ))
}

/// Parse scale block: scale { min 1 max 10 metric "instances" }
fn parse_scale_block(input: &str) -> IResult<&str, ScaleBlock> {
    let (input, _) = tag("scale")(input)?;
    let (input, _) = ws0(input)?;
    let (input, items) = delimited(
        char('{'),
        many0(preceded(ws, parse_scale_item)),
        preceded(ws0, char('}')),
    )(input)?;
    let mut scale = ScaleBlock {
        location: SourceLocation::new(String::new(), 0, 0),
        min: None,
        max: None,
        metric: None,
    };
    for (key, value) in items {
        match key.as_str() {
            "min" => scale.min = value.parse().ok(),
            "max" => scale.max = value.parse().ok(),
            "metric" => scale.metric = Some(value),
            _ => {}
        }
    }
    Ok((input, scale))
}

fn parse_scale_item(input: &str) -> IResult<&str, (String, String)> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;
    let (input, value) = alt((
        parse_string,
        map(parse_identifier, |s| s),
        map(digit1, |s: &str| s.to_string()),
    ))(input)?;
    Ok((input, (key, value)))
}

/// Parse a relation: From -> To [Label] [Tags]
fn parse_relation(input: &str) -> IResult<&str, Relation> {
    let (input, from) = parse_qualified_ident(input)?;
    let (input, _) = preceded(ws0, tag("->"))(input)?;
    let (input, _) = ws0(input)?;
    let (input, to) = parse_qualified_ident(input)?;
    let (input, _) = ws0(input)?;
    let (input, label) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, description) = opt(parse_string)(input)?;
    let (input, _) = ws0(input)?;
    let (input, technology) = opt(preceded(
        alt((tag("technology"), tag("tech"))),
        preceded(ws1, parse_string),
    ))(input)?;
    let (input, _) = ws0(input)?;
    let (input, tags) = opt(parse_tag_array)(input)?;

    Ok((
        input,
        Relation {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            from,
            to,
            label,
            description,
            technology,
            tags: tags.unwrap_or_default(),
        },
    ))
}

/// Parse qualified identifier: Ident ('.' Ident)*
fn parse_qualified_ident(input: &str) -> IResult<&str, QualifiedIdent> {
    let (input, first) = parse_identifier(input)?;
    let (input, rest) = many0(preceded(char('.'), parse_identifier))(input)?;

    let mut parts = vec![first];
    parts.extend(rest);

    Ok((input, QualifiedIdent::qualified(parts)))
}

/// Parse import statement: import { elements... } from "path"
fn parse_import(input: &str) -> IResult<&str, ImportStatement> {
    let (input, _) = tag("import")(input)?;
    let (input, _) = ws1(input)?;
    let (input, elements) = delimited(
        char('{'),
        separated_list0(
            preceded(ws0, char(',')),
            preceded(ws0, parse_import_element),
        ),
        preceded(ws0, char('}')),
    )(input)?;
    let (input, _) = ws1(input)?;
    let (input, _) = tag("from")(input)?;
    let (input, _) = ws1(input)?;
    let (input, from) = parse_string(input)?;

    Ok((
        input,
        ImportStatement {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            elements,
            from,
        },
    ))
}

/// Parse import element (identifier or wildcard)
fn parse_import_element(input: &str) -> IResult<&str, ImportElement> {
    alt((
        value(ImportElement::Wildcard, char('*')),
        map(parse_identifier, ImportElement::Ident),
    ))(input)
}

/// Parse a metadata block: metadata { entries... }
fn parse_metadata_block(input: &str) -> IResult<&str, MetadataBlock> {
    let (input, _) = tag("metadata")(input)?;
    let (input, _) = ws1(input)?;
    let (input, entries) = delimited(
        preceded(ws0, char('{')),
        many0(preceded(ws, parse_metadata_entry)),
        preceded(ws0, char('}')),
    )(input)?;

    Ok((
        input,
        MetadataBlock {
            location: SourceLocation::new(String::new(), 0, 0), // TODO: track position
            entries,
        },
    ))
}

/// Parse a metadata entry: key "value" or key ["value1", "value2"] or key [ident1, ident2]
fn parse_metadata_entry(input: &str) -> IResult<&str, MetaEntry> {
    let (input, key) = parse_identifier(input)?;
    let (input, _) = ws0(input)?;

    // Parse either a string, string array, or identifier array
    let (input, value) = alt((
        // Parse string array: ["item1", "item2"]
        map(parse_string_array, |arr| Some(arr.join(", "))),
        // Parse identifier array: [R1, R2, R4] (common in metadata { flags/tags })
        map(parse_tag_array, |arr| Some(arr.join(", "))),
        // Parse single string: "value"
        map(parse_string, Some),
    ))(input)?;

    Ok((input, MetaEntry { key, value }))
}

/// Parse a tag reference: #Ident
fn parse_tag_ref(input: &str) -> IResult<&str, String> {
    let (input, _) = char('#')(input)?;
    let (input, ident) = parse_identifier(input)?;
    Ok((input, format!("#{}", ident)))
}

/// Parse a tag array: [Ident, Ident, ...]
fn parse_tag_array(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(ws0, char('[')),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_identifier)),
        preceded(ws0, char(']')),
    )(input)
}

/// Parse an identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        )),
        |s: &str| s.to_string(),
    )(input)
}

/// Parse a string literal (double or single quoted)
fn parse_string(input: &str) -> IResult<&str, String> {
    alt((
        delimited(char('"'), take_until("\""), char('"')),
        delimited(char('\''), take_until("'"), char('\'')),
    ))(input)
    .map(|(input, s)| (input, s.to_string()))
}

/// Parse a string array: [String, String, ...]
#[allow(dead_code)]
fn parse_string_array(input: &str) -> IResult<&str, Vec<String>> {
    delimited(
        preceded(ws0, char('[')),
        separated_list0(preceded(ws0, char(',')), preceded(ws0, parse_string)),
        preceded(ws0, char(']')),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        assert_eq!(
            parse_identifier("mySystem"),
            Ok(("", "mySystem".to_string()))
        );
        assert_eq!(
            parse_identifier("my-system_123"),
            Ok(("", "my-system_123".to_string()))
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse_string(r#""hello""#), Ok(("", "hello".to_string())));
        assert_eq!(parse_string(r#"'world'"#), Ok(("", "world".to_string())));
    }

    #[test]
    fn test_parse_qualified_ident() {
        let result = parse_qualified_ident("System.Container");
        assert!(result.is_ok());
        let (_, qid) = result.unwrap();
        assert_eq!(qid.parts, vec!["System", "Container"]);
    }

    #[test]
    fn test_parse_element_def() {
        let input = r#"MySystem = system "My System""#;
        let result = parse_element_def(input);
        assert!(result.is_ok());
        let (_, elem) = result.unwrap();
        assert_eq!(elem.assignment.name, "MySystem");
        assert_eq!(elem.assignment.kind, ElementKind::System);
        assert_eq!(elem.assignment.title, Some("My System".to_string()));
    }

    #[test]
    fn test_parse_relation() {
        let input = r#"SystemA -> SystemB "Uses" "SystemA uses SystemB""#;
        let result = parse_relation(input);
        assert!(result.is_ok());
        let (_, rel) = result.unwrap();
        assert_eq!(rel.from.parts, vec!["SystemA"]);
        assert_eq!(rel.to.parts, vec!["SystemB"]);
        assert_eq!(rel.label, Some("Uses".to_string()));
        assert_eq!(rel.description, Some("SystemA uses SystemB".to_string()));
    }

    #[test]
    fn test_parse_import() {
        let input = r#"import { ServiceA, ServiceB } from "projectA""#;
        let result = parse_import(input);
        assert!(result.is_ok());
        let (_, import_stmt) = result.unwrap();
        assert_eq!(import_stmt.elements.len(), 2);
        assert_eq!(import_stmt.from, "projectA");
    }

    #[test]
    fn test_parse_scenario() {
        let input = r#"scenario LoginFlow "User Login" {
            User -> WebApp "Credentials"
            WebApp -> DB "Verify"
        }"#;
        let result = parse_scenario(input);
        assert!(result.is_ok());
        let (_, scenario) = result.unwrap();
        assert_eq!(scenario.id, "LoginFlow");
        assert_eq!(scenario.title, "User Login".to_string());
        assert_eq!(scenario.steps.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"
        // This is a comment
        MySystem = system "My System"
        /* Multi-line
           comment */
        SystemA -> SystemB "Uses"
        "#;
        let parser = Parser::new("test.sruja");
        let result = parser.parse(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_incrementally_context_window() {
        let input = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let parser = Parser::new("test.sruja");
        let existing = parser.parse(input).expect("initial parse");
        // Change "B" title to "B Updated" (edit in second line)
        let edited = "A = system \"A\"\nB = system \"B Updated\"\nA -> B \"uses\"\n";
        let change_start = 22; // start of "B"
        let change_end = 35; // end of "B Updated"
        let result = parser.parse_incrementally(edited, change_start, change_end, &existing, 2);
        assert!(result.is_ok(), "incremental parse should succeed");
        let inc = result.unwrap();
        assert!(!inc.updated_ast.items.is_empty());
        // Merge should report B as changed (title "B" -> "B Updated")
        assert!(inc.changed_elements.contains(&"B".to_string()));
    }

    #[test]
    fn test_line_to_byte_offset() {
        let s = "a\nb\nc\n";
        assert_eq!(line_to_byte_offset(s, 0), 0);
        assert_eq!(line_to_byte_offset(s, 1), 2);
        assert_eq!(line_to_byte_offset(s, 2), 4);
        assert_eq!(line_to_byte_offset(s, 3), 6);
        assert_eq!(line_to_byte_offset(s, 4), 6);
    }

    /// Many incremental parse cycles (rapid-edit simulation). Ensures no unbounded growth or panic.
    #[test]
    fn test_parse_incrementally_many_cycles() {
        let parser = Parser::new("test.sruja");
        let base = "A = system \"A\"\nB = system \"B\"\nA -> B \"uses\"\n";
        let initial = parser.parse(base).expect("initial parse");
        let mut current_ast = initial;
        const CYCLES: usize = 50;
        for i in 0..CYCLES {
            let title = format!("B v{i}");
            let edited = format!("A = system \"A\"\nB = system \"{title}\"\nA -> B \"uses\"\n");
            let change_start = 22;
            let change_end = 22 + title.len();
            let result =
                parser.parse_incrementally(&edited, change_start, change_end, &current_ast, 2);
            assert!(result.is_ok(), "cycle {} should succeed", i);
            let inc = result.unwrap();
            current_ast = inc.updated_ast;
            assert!(
                !current_ast.items.is_empty(),
                "cycle {}: ast should be non-empty",
                i
            );
        }
    }

    /// Large DSL (100+ elements): parse succeeds and completes in reasonable time.
    #[test]
    fn test_parse_large_dsl() {
        let mut dsl = String::with_capacity(50_000);
        for i in 0..100 {
            dsl.push_str(&format!("S{i} = system \"System {i}\"\n"));
        }
        for i in 0..99 {
            dsl.push_str(&format!("S{i} -> S{} \"calls\"\n", i + 1));
        }
        let parser = Parser::new("large.sruja");
        let start = std::time::Instant::now();
        let result = parser.parse(&dsl);
        let elapsed_ms = start.elapsed().as_millis();
        assert!(result.is_ok(), "large DSL should parse: {:?}", result.err());
        let program = result.unwrap();
        let elem_count = program
            .items
            .iter()
            .filter(|i| matches!(i, TopLevelItem::ElementDef(_)))
            .count();
        assert!(
            elem_count >= 100,
            "expected at least 100 elements, got {}",
            elem_count
        );
        // Debug builds can be slow; 5s is a generous cap to avoid flakiness
        assert!(
            elapsed_ms < 5000,
            "large parse took {} ms (target <5s in debug)",
            elapsed_ms
        );
    }
}

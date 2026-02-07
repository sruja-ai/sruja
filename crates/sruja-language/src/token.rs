//! Token types and definitions for Sruja DSL

use serde::{Deserialize, Serialize};

/// Token type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    // Special
    Eof,
    Illegal,

    // Identifiers + Literals
    Ident,
    String,
    Number,

    // Operators
    Assign,    // =
    Arrow,     // ->
    BackArrow, // <-
    BiArrow,   // <->

    // Delimiters
    Colon,    // :
    Comma,    // ,
    Dot,      // .
    Star,     // *
    TagRef,   // #tagname
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Structure Keywords
    Specification,
    Model,
    Views,
    View,

    // View Predicates
    Include,
    Exclude,
    Of,
    Title,
    Import,
    From,

    // Element Types
    Element,
    Person,
    System,
    Container,
    Component,
    Database,
    Queue,

    // Properties
    Description,
    Technology,
    Metadata,

    // Booleans
    True,
    False,

    // Flow Keywords
    Step,
    Flow,
    Actor,
    Kind,

    // Sruja-specific keywords
    Relation,
    Functional,
    Nonfunctional,
    Constraint,
    Story,
    Scenario,
    Requirement,
    Adr,
    Style,
    Styles,
    Properties,
    Tech,

    // Legacy (not used in parser)
    Requirements,
    Adrs,
}

impl TokenType {
    /// Convert a token type to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Eof => "EOF",
            TokenType::Illegal => "ILLEGAL",
            TokenType::Ident => "IDENT",
            TokenType::String => "STRING",
            TokenType::Number => "NUMBER",
            TokenType::Assign => "=",
            TokenType::Arrow => "->",
            TokenType::BackArrow => "<-",
            TokenType::BiArrow => "<->",
            TokenType::Colon => ":",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Star => "*",
            TokenType::TagRef => "TAG_REF",
            TokenType::LBrace => "{",
            TokenType::RBrace => "}",
            TokenType::LBracket => "[",
            TokenType::RBracket => "]",
            TokenType::Specification => "SPECIFICATION",
            TokenType::Model => "MODEL",
            TokenType::Views => "VIEWS",
            TokenType::View => "VIEW",
            TokenType::Include => "INCLUDE",
            TokenType::Exclude => "EXCLUDE",
            TokenType::Of => "OF",
            TokenType::Title => "TITLE",
            TokenType::Import => "IMPORT",
            TokenType::From => "FROM",
            TokenType::Element => "ELEMENT",
            TokenType::Person => "PERSON",
            TokenType::System => "SYSTEM",
            TokenType::Container => "CONTAINER",
            TokenType::Component => "COMPONENT",
            TokenType::Database => "DATABASE",
            TokenType::Queue => "QUEUE",
            TokenType::Description => "DESCRIPTION",
            TokenType::Technology => "TECHNOLOGY",
            TokenType::Metadata => "METADATA",
            TokenType::True => "TRUE",
            TokenType::False => "FALSE",
            TokenType::Step => "STEP",
            TokenType::Flow => "FLOW",
            TokenType::Actor => "ACTOR",
            TokenType::Kind => "KIND",
            TokenType::Relation => "RELATION",
            TokenType::Functional => "FUNCTIONAL",
            TokenType::Nonfunctional => "NONFUNCTIONAL",
            TokenType::Constraint => "CONSTRAINT",
            TokenType::Story => "STORY",
            TokenType::Scenario => "SCENARIO",
            TokenType::Requirement => "REQUIREMENT",
            TokenType::Adr => "ADR",
            TokenType::Style => "STYLE",
            TokenType::Styles => "STYLES",
            TokenType::Properties => "PROPERTIES",
            TokenType::Tech => "TECH",
            TokenType::Requirements => "REQUIREMENTS",
            TokenType::Adrs => "ADRS",
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Lookup identifier to see if it's a keyword
pub fn lookup_ident(ident: &str) -> TokenType {
    match ident {
        "specification" => TokenType::Specification,
        "model" => TokenType::Model,
        "views" => TokenType::Views,
        "view" => TokenType::View,
        "include" => TokenType::Include,
        "exclude" => TokenType::Exclude,
        "of" => TokenType::Of,
        "title" => TokenType::Title,
        "import" => TokenType::Import,
        "from" => TokenType::From,
        "element" => TokenType::Element,
        "person" => TokenType::Person,
        "system" => TokenType::System,
        "container" => TokenType::Container,
        "component" => TokenType::Component,
        "database" => TokenType::Database,
        "queue" => TokenType::Queue,
        "description" => TokenType::Description,
        "technology" => TokenType::Technology,
        "metadata" => TokenType::Metadata,
        "true" => TokenType::True,
        "false" => TokenType::False,
        "functional" => TokenType::Functional,
        "nonfunctional" => TokenType::Nonfunctional,
        "constraint" => TokenType::Constraint,
        "story" => TokenType::Story,
        "scenario" => TokenType::Scenario,
        "requirement" => TokenType::Requirement,
        "adr" => TokenType::Adr,
        "style" => TokenType::Style,
        "styles" => TokenType::Styles,
        "properties" => TokenType::Properties,
        "tech" => TokenType::Tech,
        "step" => TokenType::Step,
        "flow" => TokenType::Flow,
        "actor" => TokenType::Actor,
        "kind" => TokenType::Kind,
        "requirements" => TokenType::Requirements,
        "adrs" => TokenType::Adrs,
        _ => TokenType::Ident,
    }
}

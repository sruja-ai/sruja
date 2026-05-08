//! Token types and definitions for Sruja DSL

use serde::{Deserialize, Serialize};

/// Token type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    // Special
    /// End of File
    Eof,
    /// Illegal/unrecognized character sequence
    Illegal,

    // Identifiers + Literals
    /// An identifier (e.g. name of a system, component, container)
    Ident,
    /// A double-quoted string literal
    String,
    /// A number literal
    Number,

    // Operators
    /// Assignment operator `=`
    Assign, // =
    /// Outgoing relationship arrow `->`
    Arrow, // ->
    /// Incoming relationship arrow `<-`
    BackArrow, // <-
    /// Bidirectional relationship arrow `<->`
    BiArrow, // <->

    // Delimiters
    /// Colon delimiter `:`
    Colon, // :
    /// Comma delimiter `,`
    Comma, // ,
    /// Dot delimiter `.` for nested references
    Dot, // .
    /// Star/wildcard `*`
    Star, // *
    /// Tag reference (e.g., `#tagname`)
    TagRef, // #tagname
    /// Left curly brace `{`
    LBrace, // {
    /// Right curly brace `}`
    RBrace, // }
    /// Left bracket `[`
    LBracket, // [
    /// Right bracket `]`
    RBracket, // ]

    // Structure Keywords
    /// `specification` keyword
    Specification,
    /// `model` keyword
    Model,
    /// `views` keyword
    Views,
    /// `view` keyword
    View,

    // View Predicates
    /// `include` keyword
    Include,
    /// `exclude` keyword
    Exclude,
    /// `of` keyword
    Of,
    /// `title` keyword
    Title,
    /// `import` keyword
    Import,
    /// `from` keyword
    From,

    // Element Types
    /// `element` keyword
    Element,
    /// `person` keyword
    Person,
    /// `system` keyword
    System,
    /// `container` keyword
    Container,
    /// `component` keyword
    Component,
    /// `database` keyword
    Database,
    /// `queue` keyword
    Queue,

    // Properties
    /// `description` keyword
    Description,
    /// `technology` keyword
    Technology,
    /// `metadata` keyword
    Metadata,

    // Booleans
    /// `true` boolean keyword
    True,
    /// `false` boolean keyword
    False,

    // Flow Keywords
    /// `step` keyword
    Step,
    /// `flow` keyword
    Flow,
    /// `actor` keyword
    Actor,
    /// `kind` keyword
    Kind,

    // Sruja-specific keywords
    /// `relation` keyword
    Relation,
    /// `functional` keyword
    Functional,
    /// `nonfunctional` keyword
    Nonfunctional,
    /// `constraint` keyword
    Constraint,
    /// `story` keyword
    Story,
    /// `scenario` keyword
    Scenario,
    /// `requirement` keyword
    Requirement,
    /// `adr` keyword
    Adr,
    /// `style` keyword
    Style,
    /// `styles` keyword
    Styles,
    /// `properties` keyword
    Properties,
    /// `tech` keyword
    Tech,

    // Legacy (not used in parser)
    /// `requirements` keyword
    Requirements,
    /// `adrs` keyword
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
        "relation" => TokenType::Relation,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_type_as_str_special_tokens() {
        assert_eq!(TokenType::Eof.as_str(), "EOF");
        assert_eq!(TokenType::Illegal.as_str(), "ILLEGAL");
    }

    #[test]
    fn test_token_type_as_str_identifiers_literals() {
        assert_eq!(TokenType::Ident.as_str(), "IDENT");
        assert_eq!(TokenType::String.as_str(), "STRING");
        assert_eq!(TokenType::Number.as_str(), "NUMBER");
    }

    #[test]
    fn test_token_type_as_str_operators() {
        assert_eq!(TokenType::Assign.as_str(), "=");
        assert_eq!(TokenType::Arrow.as_str(), "->");
        assert_eq!(TokenType::BackArrow.as_str(), "<-");
        assert_eq!(TokenType::BiArrow.as_str(), "<->");
    }

    #[test]
    fn test_token_type_as_str_delimiters() {
        assert_eq!(TokenType::Colon.as_str(), ":");
        assert_eq!(TokenType::Comma.as_str(), ",");
        assert_eq!(TokenType::Dot.as_str(), ".");
        assert_eq!(TokenType::Star.as_str(), "*");
        assert_eq!(TokenType::TagRef.as_str(), "TAG_REF");
        assert_eq!(TokenType::LBrace.as_str(), "{");
        assert_eq!(TokenType::RBrace.as_str(), "}");
        assert_eq!(TokenType::LBracket.as_str(), "[");
        assert_eq!(TokenType::RBracket.as_str(), "]");
    }

    #[test]
    fn test_token_type_as_str_structure_keywords() {
        assert_eq!(TokenType::Specification.as_str(), "SPECIFICATION");
        assert_eq!(TokenType::Model.as_str(), "MODEL");
        assert_eq!(TokenType::Views.as_str(), "VIEWS");
        assert_eq!(TokenType::View.as_str(), "VIEW");
    }

    #[test]
    fn test_token_type_as_str_view_predicates() {
        assert_eq!(TokenType::Include.as_str(), "INCLUDE");
        assert_eq!(TokenType::Exclude.as_str(), "EXCLUDE");
        assert_eq!(TokenType::Of.as_str(), "OF");
        assert_eq!(TokenType::Title.as_str(), "TITLE");
        assert_eq!(TokenType::Import.as_str(), "IMPORT");
        assert_eq!(TokenType::From.as_str(), "FROM");
    }

    #[test]
    fn test_token_type_as_str_element_types() {
        assert_eq!(TokenType::Element.as_str(), "ELEMENT");
        assert_eq!(TokenType::Person.as_str(), "PERSON");
        assert_eq!(TokenType::System.as_str(), "SYSTEM");
        assert_eq!(TokenType::Container.as_str(), "CONTAINER");
        assert_eq!(TokenType::Component.as_str(), "COMPONENT");
        assert_eq!(TokenType::Database.as_str(), "DATABASE");
        assert_eq!(TokenType::Queue.as_str(), "QUEUE");
    }

    #[test]
    fn test_token_type_as_str_properties() {
        assert_eq!(TokenType::Description.as_str(), "DESCRIPTION");
        assert_eq!(TokenType::Technology.as_str(), "TECHNOLOGY");
        assert_eq!(TokenType::Metadata.as_str(), "METADATA");
    }

    #[test]
    fn test_token_type_as_str_booleans() {
        assert_eq!(TokenType::True.as_str(), "TRUE");
        assert_eq!(TokenType::False.as_str(), "FALSE");
    }

    #[test]
    fn test_token_type_as_str_flow_keywords() {
        assert_eq!(TokenType::Step.as_str(), "STEP");
        assert_eq!(TokenType::Flow.as_str(), "FLOW");
        assert_eq!(TokenType::Actor.as_str(), "ACTOR");
        assert_eq!(TokenType::Kind.as_str(), "KIND");
    }

    #[test]
    fn test_token_type_as_str_sruja_keywords() {
        assert_eq!(TokenType::Relation.as_str(), "RELATION");
        assert_eq!(TokenType::Functional.as_str(), "FUNCTIONAL");
        assert_eq!(TokenType::Nonfunctional.as_str(), "NONFUNCTIONAL");
        assert_eq!(TokenType::Constraint.as_str(), "CONSTRAINT");
        assert_eq!(TokenType::Story.as_str(), "STORY");
        assert_eq!(TokenType::Scenario.as_str(), "SCENARIO");
        assert_eq!(TokenType::Requirement.as_str(), "REQUIREMENT");
        assert_eq!(TokenType::Adr.as_str(), "ADR");
        assert_eq!(TokenType::Style.as_str(), "STYLE");
        assert_eq!(TokenType::Styles.as_str(), "STYLES");
        assert_eq!(TokenType::Properties.as_str(), "PROPERTIES");
        assert_eq!(TokenType::Tech.as_str(), "TECH");
    }

    #[test]
    fn test_lookup_ident_structure_keywords() {
        assert_eq!(lookup_ident("specification"), TokenType::Specification);
        assert_eq!(lookup_ident("model"), TokenType::Model);
        assert_eq!(lookup_ident("views"), TokenType::Views);
        assert_eq!(lookup_ident("view"), TokenType::View);
    }

    #[test]
    fn test_lookup_ident_view_predicates() {
        assert_eq!(lookup_ident("include"), TokenType::Include);
        assert_eq!(lookup_ident("exclude"), TokenType::Exclude);
        assert_eq!(lookup_ident("of"), TokenType::Of);
        assert_eq!(lookup_ident("title"), TokenType::Title);
        assert_eq!(lookup_ident("import"), TokenType::Import);
        assert_eq!(lookup_ident("from"), TokenType::From);
    }

    #[test]
    fn test_lookup_ident_element_types() {
        assert_eq!(lookup_ident("element"), TokenType::Element);
        assert_eq!(lookup_ident("person"), TokenType::Person);
        assert_eq!(lookup_ident("system"), TokenType::System);
        assert_eq!(lookup_ident("container"), TokenType::Container);
        assert_eq!(lookup_ident("component"), TokenType::Component);
        assert_eq!(lookup_ident("database"), TokenType::Database);
        assert_eq!(lookup_ident("queue"), TokenType::Queue);
    }

    #[test]
    fn test_lookup_ident_properties() {
        assert_eq!(lookup_ident("description"), TokenType::Description);
        assert_eq!(lookup_ident("technology"), TokenType::Technology);
        assert_eq!(lookup_ident("metadata"), TokenType::Metadata);
    }

    #[test]
    fn test_lookup_ident_booleans() {
        assert_eq!(lookup_ident("true"), TokenType::True);
        assert_eq!(lookup_ident("false"), TokenType::False);
    }

    #[test]
    fn test_lookup_ident_flow_keywords() {
        assert_eq!(lookup_ident("step"), TokenType::Step);
        assert_eq!(lookup_ident("flow"), TokenType::Flow);
        assert_eq!(lookup_ident("actor"), TokenType::Actor);
        assert_eq!(lookup_ident("kind"), TokenType::Kind);
    }

    #[test]
    fn test_lookup_ident_sruja_keywords() {
        assert_eq!(lookup_ident("relation"), TokenType::Relation);
        assert_eq!(lookup_ident("functional"), TokenType::Functional);
        assert_eq!(lookup_ident("nonfunctional"), TokenType::Nonfunctional);
        assert_eq!(lookup_ident("constraint"), TokenType::Constraint);
        assert_eq!(lookup_ident("story"), TokenType::Story);
        assert_eq!(lookup_ident("scenario"), TokenType::Scenario);
        assert_eq!(lookup_ident("requirement"), TokenType::Requirement);
        assert_eq!(lookup_ident("adr"), TokenType::Adr);
        assert_eq!(lookup_ident("style"), TokenType::Style);
        assert_eq!(lookup_ident("styles"), TokenType::Styles);
        assert_eq!(lookup_ident("properties"), TokenType::Properties);
        assert_eq!(lookup_ident("tech"), TokenType::Tech);
    }

    #[test]
    fn test_lookup_ident_legacy_keywords() {
        assert_eq!(lookup_ident("requirements"), TokenType::Requirements);
        assert_eq!(lookup_ident("adrs"), TokenType::Adrs);
    }

    #[test]
    fn test_lookup_ident_non_keywords_return_ident() {
        assert_eq!(lookup_ident("MySystem"), TokenType::Ident);
        assert_eq!(lookup_ident("API_Container"), TokenType::Ident);
        assert_eq!(lookup_ident("custom123"), TokenType::Ident);
        assert_eq!(lookup_ident("unknown_keyword"), TokenType::Ident);
        assert_eq!(lookup_ident(""), TokenType::Ident);
    }

    #[test]
    fn test_lookup_ident_case_sensitive() {
        assert_eq!(lookup_ident("system"), TokenType::System);
        assert_eq!(lookup_ident("System"), TokenType::Ident);
        assert_eq!(lookup_ident("SYSTEM"), TokenType::Ident);
        assert_eq!(lookup_ident("true"), TokenType::True);
        assert_eq!(lookup_ident("True"), TokenType::Ident);
    }

    #[test]
    fn test_token_type_display() {
        assert_eq!(format!("{}", TokenType::System), "SYSTEM");
        assert_eq!(format!("{}", TokenType::Arrow), "->");
        assert_eq!(format!("{}", TokenType::Ident), "IDENT");
        assert_eq!(format!("{}", TokenType::True), "TRUE");
    }
}

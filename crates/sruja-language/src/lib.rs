//! Sruja Language Processing Library
//!
//! This crate provides the core language infrastructure for the Sruja DSL,
//! including parsing, AST representation, and architectural traversal.
//!

#![warn(missing_docs)]
//! ## DSL Structure
//!
//! The Sruja DSL follows a hierarchical C4-style structure:
//! - **System:** A top-level software system or external service.
//! - **Container:** A deployable unit (service, database, etc.) nested within a system.
//! - **Component:** A logical building block nested within a container.
//! - **Person:** A human actor interacting with the systems.
//!
//! ## Crate Components
//!
//! - [`ast`]: Abstract Syntax Tree definitions representing the Sruja DSL.
//! - [`parser`]: Recursive descent parser that converts DSL strings into an AST.
//! - [`traversal`]: Utilities for walking the AST and resolving element references (FQDNs).
//! - [`schema`]: Domain-specific validation schemas (e.g., Architecture, Threat Model).
//! - [`token`]: Lexical tokens used by the parser.

#[allow(missing_docs)]
pub mod ast;
pub mod parser;
pub mod schema;
pub mod token;
pub mod traversal;

pub use ast::*;
pub use parser::Parser;
pub use schema::DomainSchema;
pub use traversal::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_system() {
        let input = r#"
MySystem = system "My System" {
  description "A test system"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse simple system");
    }

    #[test]
    fn test_parse_relation() {
        let input = r#"
A = system "System A"
B = system "System B"
A -> B "calls"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse relation");
    }

    /// Regression guard: platform architecture doc should parse quickly on developer hardware.
    /// Uses the checked-in `docs/architecture/sruja-platform.sruja` fixture (median of 20 runs).
    #[test]
    fn perf_parse_docs_platform_under_100ms() {
        use std::path::PathBuf;
        use std::time::{Duration, Instant};

        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = repo_root.join("docs/architecture/sruja-platform.sruja");
        let input =
            std::fs::read_to_string(path).expect("read docs/architecture/sruja-platform.sruja");

        let parser = Parser::new("docs/architecture/sruja-platform.sruja".to_string());

        for _ in 0..3 {
            parser.parse(&input).expect("parse warmup");
        }

        let mut samples: Vec<Duration> = Vec::with_capacity(20);
        for _ in 0..20 {
            let start = Instant::now();
            parser.parse(&input).expect("parse");
            samples.push(start.elapsed());
        }

        samples.sort();
        let median = samples[samples.len() / 2];
        assert!(
            median < Duration::from_millis(100),
            "median parse time was {median:?}"
        );
    }

    #[test]
    fn test_parse_nested_elements() {
        let input = r#"
MySystem = system "My System" {
  API = container "API Container" {
    Handler = component "Request Handler"
  }
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse nested elements");
    }

    #[test]
    fn test_fqdn_edge_resolution() {
        let input = r#"
Backend = system "Backend System" {
    API = container "API Service"
    DB = container "Database"

    API -> DB "reads/writes"
}

Frontend = system "Frontend System" {
    WebApp = container "Web Application"
}

Frontend.WebApp -> Backend.API "calls API"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse system with nested relations");

        let program = result.unwrap();
        let (elements, relations) = collect_elements(&program);

        // Check that elements are stored with FQDNs
        assert!(
            elements.contains_key("Backend"),
            "Backend system should exist"
        );
        assert!(
            elements.contains_key("Backend.API"),
            "Backend.API container should exist"
        );
        assert!(
            elements.contains_key("Backend.DB"),
            "Backend.DB container should exist"
        );
        assert!(
            elements.contains_key("Frontend"),
            "Frontend system should exist"
        );
        assert!(
            elements.contains_key("Frontend.WebApp"),
            "Frontend.WebApp container should exist"
        );

        // Check that relations have resolved FQDNs
        assert_eq!(relations.len(), 2, "Should have 2 relations");

        // Find the relation: API -> DB (defined inside Backend body)
        let rel1 = relations
            .iter()
            .find(|r| r.label.as_deref() == Some("reads/writes"))
            .expect("Should find 'reads/writes' relation");
        assert_eq!(
            rel1.from.as_string(),
            "Backend.API",
            "Relation 'reads/writes' from should be resolved to Backend.API"
        );
        assert_eq!(
            rel1.to.as_string(),
            "Backend.DB",
            "Relation 'reads/writes' to should be resolved to Backend.DB"
        );

        // Find the relation: Frontend.WebApp -> Backend.API (defined with FQDN)
        let rel2 = relations
            .iter()
            .find(|r| r.label.as_deref() == Some("calls API"))
            .expect("Should find 'calls API' relation");
        assert_eq!(
            rel2.from.as_string(),
            "Frontend.WebApp",
            "Relation 'calls API' from should be Frontend.WebApp"
        );
        assert_eq!(
            rel2.to.as_string(),
            "Backend.API",
            "Relation 'calls API' to should be Backend.API"
        );
    }

    #[test]
    fn test_parse_nested_containers_under_system() {
        // Minimal case: system with multiple nested containers (matches pattern_microservices structure)
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"
component = kind "Component"
database = kind "Database"
queue = kind "Queue"

customer = person "Customer" {
  description "End user purchasing products"
  metadata { tags ["R1", "R2"] }
}

merchant = person "Merchant" {
  description "Seller managing inventory"
  metadata { tags ["R1"] }
}

admin = person "Platform Administrator" {
  description "Admin managing the platform"
  metadata { tags ["R3"] }
}

ecommerce = system "E-Commerce Platform" {
  metadata { flags ["R1", "R4"] }
  apiGateway = container "API Gateway" {
    technology "Kong"
  }
  catalogService = container "Catalog Service" {
    technology "Java, Spring Boot"
    description "Manages product catalog information"

    productApi = component "Product API" {
      description "API for product retrieval"
      technology "Spring MVC"
      scale { min 2 max 5 metric "request rate > 1000 req/s" }
      tags ["R1", "R2"]
    }
    searchApi = component "Search API" {
        description "Full-text search for products"
        technology "Elasticsearch Client"
        tags ["R4"]
    }
    categoryApi = component "Category API" {
        description "Manages product categories"
        technology "Spring MVC"
    }
  }
  inventoryService = container "Inventory Service" {
    technology "Go"
    stockApi = component "Stock API" {}
  }
  cartService = container "Cart Service" {
    technology "Node.js"
  }
  catalogDb = database "Catalog Database" {
    technology "PostgreSQL"
    metadata { tags ["R1"] }
  }
}

paymentProvider = system "Payment Provider" {
  description "Third-party payment gateway"
  metadata {
    tags ["external", "R3"]
  }
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse");

        let program = result.unwrap();
        let (elements, _) = collect_elements(&program);

        assert!(elements.contains_key("ecommerce"), "ecommerce");
        assert!(elements.contains_key("ecommerce.apiGateway"), "apiGateway");
        assert!(
            elements.contains_key("ecommerce.catalogService"),
            "catalogService"
        );
        assert!(
            elements.contains_key("ecommerce.catalogService.productApi"),
            "productApi"
        );
        assert!(
            elements.contains_key("ecommerce.inventoryService"),
            "inventoryService"
        );
        assert!(
            elements.contains_key("ecommerce.inventoryService.stockApi"),
            "stockApi"
        );
    }

    #[test]
    fn test_parse_pattern_microservices_collects_nested_elements() {
        let content = include_str!("../../../book/valid-examples/pattern-microservices.sruja");
        let parser = Parser::new("book/valid-examples/pattern-microservices.sruja".to_string());
        let result = parser.parse(content);
        assert!(result.is_ok(), "pattern_microservices.sruja should parse");

        let program = result.unwrap();
        let (elements, _) = collect_elements(&program);

        // Must have ecommerce system and its nested containers (relations reference ecommerce.X)
        let expected = [
            "ecommerce",
            "ecommerce.apiGateway",
            "ecommerce.catalogService",
            "ecommerce.inventoryService",
            "ecommerce.cartService",
            "ecommerce.orderService",
            "ecommerce.paymentService",
            "ecommerce.userService",
            "ecommerce.notificationService",
            "ecommerce.catalogDb",
            "ecommerce.inventoryDb",
            "ecommerce.cartDb",
            "ecommerce.orderDb",
            "ecommerce.userDb",
        ];
        for fqn in expected {
            assert!(
                elements.contains_key(fqn),
                "Expected element '{}' not found. Have: {:?}",
                fqn,
                elements.keys().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_causal_loop_qualified_relation_scoped() {
        // Relations with qualified idents (ecommerce.api -> admin) must stay inside the loop and get scope
        let input = r#"
admin = person "Admin"
ecommerce = system "E" {
  api = container "API" {}
}
AdminFeedback = causal_loop "Admin Feedback Loop" {
  loop_type "balancing"
  ecommerce.api -> admin "sends alert to"
  admin -> ecommerce.api "adjusts via"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_ok(), "Should parse: {:?}", result);

        let program = result.unwrap();
        let with_scope = collect_relations_with_scope(&program);
        let causal_scoped: Vec<_> = with_scope
            .iter()
            .filter(|rws| rws.scope == "AdminFeedback")
            .collect();
        assert!(
            causal_scoped.len() >= 2,
            "Causal loop relations should be scoped to AdminFeedback; got {:?}",
            with_scope
                .iter()
                .map(|r| (
                    r.scope.as_str(),
                    r.relation.from.as_string(),
                    r.relation.to.as_string()
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_parse_missing_brace_error() {
        let input = "MySystem = system \"My System\" {";
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Missing closing `}`"));
        assert!(errors[0].suggestions[0].contains("Add a matching `}`"));
    }

    #[test]
    fn test_parse_unterminated_string_error() {
        let input = "MySystem = system \"My System";
        let parser = Parser::new("test.sruja".to_string());
        let result = parser.parse(input);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("Unterminated string literal"));
        assert!(errors[0].suggestions[0].contains("Close the string"));
    }
}

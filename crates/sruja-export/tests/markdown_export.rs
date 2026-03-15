//! Tests for Markdown exporter functionality

#[cfg(test)]
mod tests {
    use sruja_diagnostics::SourceLocation;
    use sruja_export::markdown::MarkdownOptions;
    use sruja_export::MarkdownExporter;
    use sruja_language::{Parser, TopLevelItem};

    #[test]
    fn test_basic_system_export() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User" {
    description "End user of the system"
}

Shop = system "Shop" {
    description "Online shopping platform"

    Web = container "Web Application" {
        technology "React"
    }

    API = container "API Service" {
        technology "Rust"
    }
}

User -> Shop.Web "uses"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: true,
            include_systems: true,
            include_persons: true,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("# Table of Contents"));
        assert!(markdown.contains("## Overview"));
        assert!(markdown.contains("## Systems"));
        assert!(markdown.contains("## Stakeholders"));
        assert!(markdown.contains("Shop"));
        assert!(markdown.contains("Online shopping platform"));
        assert!(markdown.contains("Web Application"));
        assert!(markdown.contains("React"));
        assert!(markdown.contains("API Service"));
        assert!(markdown.contains("Rust"));
        assert!(markdown.contains("User"));
        assert!(markdown.contains("End user of the system"));
        assert!(markdown.contains("```mermaid"));
    }

    #[test]
    fn test_requirements_export() {
        let input = r#"
R1 = requirement functional "Users must be able to login"
R2 = requirement security "All passwords must be hashed"
R3 = requirement stability "99.9% uptime"
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: true,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Requirements"));
        assert!(markdown.contains("Users must be able to login"));
        assert!(markdown.contains("**Type:** functional"));
        assert!(markdown.contains("All passwords must be hashed"));
        assert!(markdown.contains("**Type:** security"));
        assert!(markdown.contains("99.9% uptime"));
        assert!(markdown.contains("**Type:** stability"));
    }

    #[test]
    fn test_adr_export() {
        let input = r#"
ADR-1 = adr "Use PostgreSQL as database" {
    status "Accepted"
    context "Need a relational database with ACID compliance"
    decision "Use PostgreSQL"
    consequences "Team needs training, but system is robust"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: true,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Architecture Decision Records"));
        assert!(markdown.contains("Use PostgreSQL as database"));
        assert!(markdown.contains("**Status:** Accepted"));
        assert!(markdown.contains("**Context:** Need a relational database with ACID compliance"));
        assert!(markdown.contains("**Decision:** Use PostgreSQL"));
        assert!(markdown.contains("**Consequences:** Team needs training, but system is robust"));
    }

    #[test]
    fn test_overview_policies_export() {
        let input = r#"
overview {
    summary "Demo architecture for intent and drift checks."
    audience "Developers and reviewers"
    scope "Core services and data flow"
    goals ["Clarity", "Lint and export"]
    non_goals ["Full production topology"]
    risks ["Scope creep"]
}

SecurityPolicy = policy "Enforce TLS" {
    category "security"
    enforcement "required"
    description "All traffic must use TLS 1.3."
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: true,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: false,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Overview"));
        assert!(markdown.contains("Demo architecture for intent and drift checks"));
        assert!(markdown.contains("**Audience:** Developers and reviewers"));
        assert!(markdown.contains("**Scope:** Core services and data flow"));
        assert!(markdown.contains("**Goals:**"));
        assert!(markdown.contains("**Non-goals:**"));
        assert!(markdown.contains("**Risks:**"));
        assert!(markdown.contains("## Policies"));
        assert!(markdown.contains("Enforce TLS"));
        assert!(markdown.contains("**Category:** security"));
        assert!(markdown.contains("**Enforcement:** required"));
        assert!(markdown.contains("All traffic must use TLS 1.3"));
        assert!(markdown.contains("- [Policies](#policies)"));
    }

    #[test]
    fn test_scenario_export_with_mermaid() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web"
    API = container "API"
}

scenario LoginFlow "User Login Flow" {
    User -> Shop.Web "enters credentials"
    Shop.Web -> Shop.API "sends login request"
    Shop.API -> User "returns session token"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: true,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Scenarios"));
        assert!(markdown.contains("User Login Flow"));
        assert!(markdown.contains("```mermaid"));
        assert!(markdown.contains("sequenceDiagram"));
    }

    #[test]
    fn test_feedback_loop_export() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
System = system "System"

FL1 = feedback "User Satisfaction Loop" {
    loop_type reinforcing
    description "Increased usage improves satisfaction"

    User -> System "provides feedback"
    System -> User "improves experience"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Feedback Loops"));
        assert!(markdown.contains("User Satisfaction Loop"));
        assert!(markdown.contains("**Type:** reinforcing (+)"));
        assert!(markdown.contains("Increased usage improves satisfaction"));
        assert!(markdown.contains("```mermaid"));
        assert!(markdown.contains("graph LR"));
    }

    #[test]
    fn test_causal_loop_export() {
        // Parser expects: Id = causal_loop "Title" { loop_type, variable, From -> To "label" }
        let input = r#"
Loop1 = causal_loop "Test Loop" {
    loop_type reinforcing
    variable Stock "Stock Variable"
    variable Flow "Flow Variable"
    Stock -> Flow "increases"
    Flow -> Stock "decreases"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Causal Loops"));
        assert!(markdown.contains("Test Loop"));
        assert!(markdown.contains("**Type:** reinforcing"));
        assert!(markdown.contains("```mermaid"));
        assert!(markdown.contains("graph LR"));
    }

    #[test]
    fn test_nested_hierarchy_export() {
        let input = r#"
system = kind "System"
container = kind "Container"
component = kind "Component"

Banking = system "Banking System" {
    description "Online banking platform"
    technology "Java"

    API = container "API Gateway" {
        description "REST API endpoints"
        technology "Spring Boot"

        Auth = component "Auth Service" {
            description "OAuth2 authentication"
        }

        Accounts = component "Accounts Service" {
            description "Account management"
        }
    }

    Web = container "Web App" {
        description "React SPA"
        technology "React"
    }
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: true,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Systems"));
        assert!(markdown.contains("### Banking System"));
        assert!(markdown.contains("Online banking platform"));
        assert!(markdown.contains("**Technology:** Java"));
        assert!(markdown.contains("#### Container diagram (L2)"));
        assert!(markdown.contains("##### API Gateway"));
        assert!(markdown.contains("REST API endpoints"));
        assert!(markdown.contains("**Technology:** Spring Boot"));
        assert!(markdown.contains("###### Component diagram (L3)"));
        assert!(markdown.contains("##### Web App"));
        assert!(markdown.contains("React SPA"));
        assert!(markdown.contains("**Technology:** React"));
        assert!(markdown.contains("```mermaid"));
    }

    #[test]
    fn test_empty_program() {
        let input = "";

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: true,
            include_systems: true,
            include_persons: true,
            include_requirements: true,
            include_adrs: true,
            include_scenarios: true,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(
            markdown.is_empty(),
            "Empty program should produce empty markdown"
        );
    }

    #[test]
    fn test_toc_links() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web"
}

R1 = requirement functional "Login requirement"
ADR-1 = adr "Use PostgreSQL" {
    status "Accepted"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: true,
            include_systems: true,
            include_persons: true,
            include_requirements: true,
            include_adrs: true,
            include_scenarios: false,
            include_mermaid_diagrams: false,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("# Table of Contents"));
        assert!(markdown.contains("- [Overview](#overview)"));
        assert!(markdown.contains("- [Systems](#systems)"));
        assert!(markdown.contains("- [Stakeholders](#stakeholders)"));
        assert!(markdown.contains("- [Requirements](#requirements)"));
        assert!(
            markdown.contains("- [Architecture Decision Records](#architecture-decision-records)")
        );
    }

    #[test]
    fn test_mermaid_disabled() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web"
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: false,
            include_overview: false,
            include_systems: true,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: false,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(
            !markdown.contains("```mermaid"),
            "Mermaid should be disabled"
        );
        assert!(markdown.contains("## Systems"));
        assert!(markdown.contains("Shop"));
    }

    #[test]
    fn test_section_inclusion_flags() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web"
}

R1 = requirement functional "Req"
ADR-1 = adr "ADR" {
    status "Accepted"
}

scenario LoginFlow {
    title "Login"
    steps [
        User -> Shop.Web "login"
    ]
}
"#;

        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: false,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(!markdown.contains("## Overview"));
        assert!(!markdown.contains("## Systems"));
        assert!(!markdown.contains("## Stakeholders"));
        assert!(!markdown.contains("## Requirements"));
        assert!(!markdown.contains("## Architecture Decision Records"));
        assert!(!markdown.contains("## Scenarios"));
        assert!(!markdown.contains("```mermaid"));
    }

    #[test]
    fn test_deployments_export() {
        // Deployment nodes are not parsed from DSL yet; build program with deployment items
        let input = r#"
person = kind "Person"
system = kind "System"
User = person "User"
App = system "App" {}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let mut program = parser.parse(input).expect("Failed to parse");
        let loc = SourceLocation::new(String::new(), 0, 0);
        let deployment = sruja_language::DeploymentNode {
            location: loc.clone(),
            id: "Prod".to_string(),
            label: Some("Production".to_string()),
            technology: Some("AWS".to_string()),
            children: vec![sruja_language::DeploymentNode {
                location: loc.clone(),
                id: "US-East".to_string(),
                label: Some("US East 1".to_string()),
                technology: None,
                children: vec![],
            }],
        };
        program.push_item(TopLevelItem::Deployment(deployment));

        let options = MarkdownOptions {
            include_toc: true,
            include_overview: false,
            include_systems: true,
            include_persons: true,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_deployments: true,
            include_mermaid_diagrams: false,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Deployments"));
        assert!(markdown.contains("- [Deployments](#deployments)"));
        assert!(markdown.contains("### Production"));
        assert!(markdown.contains("**Technology:** AWS"));
        assert!(markdown.contains("#### US East 1"));
    }

    #[test]
    fn test_element_metadata_export() {
        let input = r#"
system = kind "System"
container = kind "Container"

Shop = system "Shop" {
    description "E-commerce platform"
    metadata {
        owner "Platform Team"
        tier "critical"
    }
    API = container "API" {
        technology "Rust"
        metadata { version "2.0" }
    }
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: false,
            include_overview: false,
            include_systems: true,
            include_persons: false,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: false,
            include_metadata: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("**owner:** Platform Team"));
        assert!(markdown.contains("**tier:** critical"));
        assert!(markdown.contains("**version:** 2.0"));
    }

    #[test]
    fn test_export_with_view_definitions_in_dsl() {
        // Program with view definitions still exports (view-driven section when implemented)
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web App"
    API = container "API Service"
    DB = container "Database"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: false,
            include_overview: false,
            include_systems: true,
            include_persons: true,
            include_requirements: false,
            include_adrs: false,
            include_scenarios: false,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        assert!(markdown.contains("## Systems"));
        assert!(markdown.contains("Shop"));
        assert!(markdown.contains("Web App"));
        assert!(markdown.contains("```mermaid"));
    }

    #[test]
    fn test_export_captures_detail_fields() {
        // Verify export includes: scenario/flow description, requirement ID/tags, causal loop variables
        let input = r#"
R1 = requirement functional "Users must be able to login"
REQ2 = requirement security "Passwords must be hashed" {
    description "All passwords hashed with bcrypt"
    tags ["security", "auth"]
}

scenario Checkout "Checkout Flow" "User completes purchase from cart to payment" {
    User -> Shop.Cart "adds items"
    Shop.Cart -> Shop.Payment "submits order"
}

Loop1 = causal_loop "Stock and Flow" {
    loop_type reinforcing
    variable Inventory "Stock level"
    variable Orders "Order rate"
    Inventory -> Orders "increases"
    Orders -> Inventory "decreases"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");

        let options = MarkdownOptions {
            include_toc: false,
            include_overview: false,
            include_systems: false,
            include_persons: false,
            include_requirements: true,
            include_adrs: false,
            include_scenarios: true,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };

        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);

        // Requirement: ID when different from title (R1 vs "Users must...")
        assert!(
            markdown.contains("**ID:** R1"),
            "expected requirement ID R1 in markdown"
        );
        // Scenario description
        assert!(
            markdown.contains("User completes purchase from cart to payment"),
            "expected scenario description in markdown"
        );
        // Causal loop variables (id and label)
        assert!(
            markdown.contains("**Variables:**"),
            "expected causal loop Variables section"
        );
        assert!(
            markdown.contains("Inventory (Stock level)"),
            "expected variable Inventory with label in causal loop"
        );
    }

    #[test]
    fn test_escaping_special_chars_in_headings() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
Shop = system "C# API [v2]" {
    description "System with special chars in title"
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");
        let options = MarkdownOptions {
            include_overview: false,
            include_systems: true,
            include_persons: true,
            ..MarkdownOptions::default()
        };
        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);
        // Escaped: # → \#, [ → \[, ] → \]
        assert!(
            markdown.contains("C\\# API \\[v2\\]"),
            "expected escaped heading for system title"
        );
    }

    #[test]
    fn test_export_single_view() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

User = person "User"
Shop = system "Shop" {
    Web = container "Web App"
    API = container "API"
}

view api_focus {
    title "API Focus"
    description "Only API container"
    include Shop.API
}

User -> Shop.Web "uses"
Shop.Web -> Shop.API "calls"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");
        let options = MarkdownOptions {
            use_views: true,
            view_name: Some("api_focus".to_string()),
            include_toc: true,
            include_mermaid_diagrams: true,
            include_metadata: true,
            ..MarkdownOptions::default()
        };
        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);
        assert!(markdown.contains("## View"), "expected View section in single-view export");
        assert!(markdown.contains("### API Focus"), "expected view title");
        assert!(markdown.contains("```mermaid"), "expected Mermaid diagram");
        assert!(markdown.contains("Shop_API") || markdown.contains("Shop"), "expected view elements in diagram");
    }

    #[test]
    fn test_export_all_views() {
        let input = r#"
person = kind "Person"
system = kind "System"
container = kind "Container"

Shop = system "Shop" {
    Web = container "Web"
    API = container "API"
}

view api_view {
    title "API View"
    include Shop.API
}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");
        let options = MarkdownOptions {
            use_views: true,
            view_name: None,
            include_all_views: true,
            include_mermaid_diagrams: true,
            ..MarkdownOptions::default()
        };
        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);
        assert!(markdown.contains("## Custom views"), "expected Custom views section");
        assert!(markdown.contains("### API View"), "expected view title in Custom views");
    }

    #[test]
    fn test_export_view_not_found_falls_back_to_full_doc() {
        let input = r#"
system = kind "System"
Shop = system "Shop" {}
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");
        let options = MarkdownOptions {
            use_views: true,
            view_name: Some("nonexistent_view".to_string()),
            include_systems: true,
            ..MarkdownOptions::default()
        };
        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);
        assert!(markdown.contains("## Systems"), "expected fallback to full doc with Systems");
        assert!(markdown.contains("Shop"), "expected Shop in full doc");
    }

    #[test]
    fn test_relations_section_when_include_relations() {
        let input = r#"
system = kind "System"
container = kind "Container"

A = system "System A" {}
B = system "System B" {}

A -> B "calls"
"#;
        let parser = Parser::new("test.sruja".to_string());
        let program = parser.parse(input).expect("Failed to parse");
        let options = MarkdownOptions {
            include_relations: true,
            include_systems: true,
            ..MarkdownOptions::default()
        };
        let exporter = MarkdownExporter::new(options);
        let markdown = exporter.export(&program);
        assert!(markdown.contains("## Relations"), "expected Relations section");
        assert!(markdown.contains("A") && markdown.contains("B"), "expected from/to in relations");
        assert!(markdown.contains("calls"), "expected relation label");
    }
}

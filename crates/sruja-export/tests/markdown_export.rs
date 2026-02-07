//! Tests for Markdown exporter functionality

#[cfg(test)]
mod tests {
    use sruja_export::markdown::MarkdownOptions;
    use sruja_export::MarkdownExporter;
    use sruja_language::Parser;

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
        assert!(markdown.contains("## Persons"));
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
    #[ignore = "Markdown exporter does not yet emit Feedback Loops section; parser/AST may not support feedback blocks"]
    fn test_feedback_loop_export() {
        let input = r#"
person = kind "Person"
system = kind "System"

User = person "User"
System = system "System"

feedback FL-1 reinforcing "User Satisfaction Loop" "Increased usage improves satisfaction" {
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
    #[ignore = "Markdown exporter does not yet emit Causal Loops section; parser/AST may not support causal_loop blocks"]
    fn test_causal_loop_export() {
        let input = r#"
causal_loop CL-1 reinforcing "Market Dynamics" "Supply and demand balance" {
    variable demand "Demand"
    variable price "Price"
    
    demand -> price effect "increases" polarity positive
    price -> demand effect "decreases" polarity negative
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
        assert!(markdown.contains("Market Dynamics"));
        assert!(markdown.contains("**Type:** reinforcing (+)"));
        assert!(markdown.contains("Supply and demand balance"));
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
        assert!(markdown.contains("- [Persons](#persons)"));
        assert!(markdown.contains("- [Requirements](#requirements)"));
        assert!(markdown.contains("- [ADRs](#adrs)"));
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
        assert!(!markdown.contains("## Persons"));
        assert!(!markdown.contains("## Requirements"));
        assert!(!markdown.contains("## Architecture Decision Records"));
        assert!(!markdown.contains("## Scenarios"));
        assert!(!markdown.contains("```mermaid"));
    }
}

//! Scenario and flow printing.

use sruja_language::{Flow, Scenario};

pub fn print_scenario(out: &mut String, scenario: &Scenario) {
    out.push_str("scenario ");
    if !scenario.id.is_empty() {
        out.push_str(&scenario.id);
        out.push(' ');
    }
    if !scenario.title.is_empty() {
        out.push('"');
        out.push_str(&scenario.title);
        out.push('"');
        out.push(' ');
    }
    if let Some(desc) = &scenario.description {
        out.push('"');
        out.push_str(desc);
        out.push('"');
        out.push(' ');
    }
    if !scenario.steps.is_empty() {
        out.push_str("{\n");
        for step in &scenario.steps {
            if let Some(from) = &step.from {
                out.push_str(&format!("  {} -> ", from.as_string()));
            }
            if let Some(to) = &step.to {
                out.push_str(&to.as_string());
            }
            if let Some(desc) = &step.description {
                out.push_str(" \"");
                out.push_str(desc);
                out.push('"');
            }
            out.push('\n');
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}

pub fn print_flow(out: &mut String, flow: &Flow) {
    out.push_str("flow ");
    if !flow.id.is_empty() {
        out.push_str(&flow.id);
        out.push(' ');
    }
    if !flow.title.is_empty() {
        out.push('"');
        out.push_str(&flow.title);
        out.push('"');
        out.push(' ');
    }
    if let Some(desc) = &flow.description {
        out.push('"');
        out.push_str(desc);
        out.push('"');
        out.push(' ');
    }
    if !flow.steps.is_empty() {
        out.push_str("{\n");
        for step in &flow.steps {
            if let Some(from) = &step.from {
                out.push_str(&format!("  {} -> ", from.as_string()));
            }
            if let Some(to) = &step.to {
                out.push_str(&to.as_string());
            }
            if let Some(desc) = &step.description {
                out.push_str(" \"");
                out.push_str(desc);
                out.push('"');
            }
            out.push('\n');
        }
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{QualifiedIdent, ScenarioStep};

    fn create_scenario(id: &str, title: &str, steps: Vec<ScenarioStep>) -> Scenario {
        Scenario {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            steps,
        }
    }

    fn create_flow(id: &str, title: &str, steps: Vec<ScenarioStep>) -> Flow {
        Flow {
            location: SourceLocation::new(String::new(), 0, 0),
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            steps,
        }
    }

    fn create_step(from: &str, to: &str, desc: Option<&str>) -> ScenarioStep {
        ScenarioStep {
            from: Some(QualifiedIdent::simple(from.to_string())),
            to: Some(QualifiedIdent::simple(to.to_string())),
            description: desc.map(|s| s.to_string()),
            tags: vec![],
            order: None,
        }
    }

    #[test]
    fn test_print_scenario_empty() {
        let mut out = String::new();
        let scenario = create_scenario("", "", vec![]);
        print_scenario(&mut out, &scenario);
        assert!(out.starts_with("scenario"));
    }

    #[test]
    fn test_print_scenario_with_id() {
        let mut out = String::new();
        let scenario = create_scenario("login_flow", "", vec![]);
        print_scenario(&mut out, &scenario);
        assert!(out.contains("login_flow"));
    }

    #[test]
    fn test_print_scenario_with_title() {
        let mut out = String::new();
        let scenario = create_scenario("id", "User Login", vec![]);
        print_scenario(&mut out, &scenario);
        assert!(out.contains("\"User Login\""));
    }

    #[test]
    fn test_print_scenario_with_steps() {
        let mut out = String::new();
        let scenario = create_scenario(
            "id",
            "Test",
            vec![
                create_step("User", "API", Some("requests")),
                create_step("API", "DB", None),
            ],
        );
        print_scenario(&mut out, &scenario);
        assert!(out.contains("User -> API"));
        assert!(out.contains("API -> DB"));
        assert!(out.contains("requests"));
        assert!(out.contains("{"));
        assert!(out.contains("}"));
    }

    #[test]
    fn test_print_scenario_step_with_description() {
        let mut out = String::new();
        let scenario =
            create_scenario("id", "Test", vec![create_step("A", "B", Some("test desc"))]);
        print_scenario(&mut out, &scenario);
        assert!(out.contains("\"test desc\""));
    }

    #[test]
    fn test_print_flow_empty() {
        let mut out = String::new();
        let flow = create_flow("", "", vec![]);
        print_flow(&mut out, &flow);
        assert!(out.starts_with("flow"));
    }

    #[test]
    fn test_print_flow_with_id() {
        let mut out = String::new();
        let flow = create_flow("checkout", "", vec![]);
        print_flow(&mut out, &flow);
        assert!(out.contains("checkout"));
    }

    #[test]
    fn test_print_flow_with_title() {
        let mut out = String::new();
        let flow = create_flow("id", "Checkout Flow", vec![]);
        print_flow(&mut out, &flow);
        assert!(out.contains("\"Checkout Flow\""));
    }

    #[test]
    fn test_print_flow_with_steps() {
        let mut out = String::new();
        let flow = create_flow(
            "id",
            "Test",
            vec![create_step("Cart", "Payment", Some("process"))],
        );
        print_flow(&mut out, &flow);
        assert!(out.contains("Cart -> Payment"));
        assert!(out.contains("process"));
    }

    #[test]
    fn test_print_flow_step_no_description() {
        let mut out = String::new();
        let flow = create_flow("id", "Test", vec![create_step("A", "B", None)]);
        print_flow(&mut out, &flow);
        assert!(out.contains("A -> B"));
        assert!(!out.contains("\"\""));
    }
}

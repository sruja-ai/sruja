//! Properties validation rule
//!
//! Mirrors Go `PropertiesValidationRule`:
//! - Validates specific metadata properties (key/value) on elements.
//! - Uses key-specific validators and emits CODE_INVALID_PROPERTY errors.

use std::collections::HashMap;

use sruja_diagnostics::{Diagnostic, Severity};
use sruja_language::{collect_elements, ElementDef, Program};

use crate::validator::Rule;

pub struct PropertiesValidationRule;

impl Rule for PropertiesValidationRule {
    fn name(&self) -> &str {
        "Properties Validation"
    }

    fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        if program.items.is_empty() {
            return vec![];
        }

        let (elements, _relations) = collect_elements(program);
        let mut diags: Vec<Diagnostic> = Vec::with_capacity(16);

        for elem in elements.values() {
            let props = extract_props(elem);
            if props.is_empty() {
                continue;
            }
            diags.extend(validate_props_map(&props, &elem.location));
        }

        diags
    }
}

fn extract_props(elem: &ElementDef) -> HashMap<String, String> {
    let mut props: HashMap<String, String> = HashMap::new();
    let Some(body) = &elem.assignment.body else {
        return props;
    };

    for entry in &body.metadata {
        if let Some(v) = &entry.value {
            props.insert(entry.key.clone(), v.clone());
        }
    }
    props
}

fn validate_props_map(
    props: &HashMap<String, String>,
    loc: &sruja_diagnostics::SourceLocation,
) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    for (k, v) in props {
        if let Some(ok) = validate_property(k, v, props) {
            if !ok {
                let mut suggestions: Vec<String> = Vec::new();
                match k.as_str() {
                    "port" => {
                        suggestions
                            .push("Port must be a valid integer between 1 and 65535".to_string());
                        suggestions.push("Example: '8080' or '443'".to_string());
                    }
                    "version" => {
                        suggestions.push(
                            "Version should follow semantic versioning (e.g., '1.0.0', '2.1.3')"
                                .to_string(),
                        );
                    }
                    "url" => {
                        suggestions.push(
                            "URL must be a valid URL format (e.g., 'https://example.com')"
                                .to_string(),
                        );
                    }
                    _ => {
                        suggestions.push(format!("Check the expected format for property '{}'", k));
                        suggestions.push(
                            "Refer to the DSL documentation for valid property values".to_string(),
                        );
                    }
                }

                diags.push(
                    Diagnostic::new(
                        sruja_diagnostics::codes::CODE_INVALID_PROPERTY,
                        Severity::Error,
                        format!("Property '{}' has invalid value '{}'", k, v),
                        loc.clone(),
                    )
                    .with_suggestions(suggestions),
                );
            }
        }
    }

    diags
}

fn validate_property(k: &str, v: &str, props: &HashMap<String, String>) -> Option<bool> {
    Some(match k {
        "capacity.instanceType" => validate_instance_type(v, props),
        "capacity.readReplicas" => is_integer(v),
        "obs.tracing.sampleRate" => is_percentage(v),
        "compliance.pci.level" => !v.is_empty(),
        "cost.monthly.total" => is_currency(v),
        "cost.monthly.compute" => is_currency(v),
        "cost.perTransaction.average" => is_currency(v),
        _ => return None,
    })
}

fn is_integer(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn is_percentage(s: &str) -> bool {
    let s = s.trim();
    let Some(num) = s.strip_suffix('%') else {
        return false;
    };
    is_number(num)
}

fn is_currency(s: &str) -> bool {
    // Go regex: ^\$\d{1,3}(,\d{3})*(\.\d+)?$
    let s = s.trim();
    let Some(rest) = s.strip_prefix('$') else {
        return false;
    };
    let (whole, frac) = rest.split_once('.').unwrap_or((rest, ""));

    // Validate whole part: groups of 1-3 digits then optional ,### groups
    let mut parts = whole.split(',');
    let first = parts.next().unwrap_or("");
    if first.is_empty() || first.len() > 3 || !first.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    for p in parts {
        if p.len() != 3 || !p.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }

    if frac.is_empty() {
        return true;
    }
    frac.chars().all(|c| c.is_ascii_digit())
}

fn validate_instance_type(s: &str, props: &HashMap<String, String>) -> bool {
    // Go uses provider-specific regexes; we implement equivalent checks without regex deps.
    let provider = props
        .get("capacity.instanceProvider")
        .map(|s| s.as_str())
        .unwrap_or("");
    match provider {
        "aws" => is_aws_instance_type(s),
        "gcp" => is_gcp_instance_type(s),
        "azure" => is_azure_instance_type(s),
        _ => !s.trim().is_empty(),
    }
}

fn is_aws_instance_type(s: &str) -> bool {
    // Regex: ^[a-z][0-9][a-z]?\.(?:nano|micro|small|medium|large|xlarge|\d+xlarge)$
    let s = s.trim();
    let Some((family, size)) = s.split_once('.') else {
        return false;
    };
    let fam: Vec<char> = family.chars().collect();
    if fam.len() < 2 || fam.len() > 3 {
        return false;
    }
    if !fam[0].is_ascii_lowercase() || !fam[1].is_ascii_digit() {
        return false;
    }
    if fam.len() == 3 && !fam[2].is_ascii_lowercase() {
        return false;
    }
    match size {
        "nano" | "micro" | "small" | "medium" | "large" | "xlarge" => true,
        _ => {
            // \d+xlarge
            let Some(num) = size.strip_suffix("xlarge") else {
                return false;
            };
            let num = num.trim_end_matches('x').trim();
            !num.is_empty() && num.chars().all(|c| c.is_ascii_digit())
        }
    }
}

fn is_gcp_instance_type(s: &str) -> bool {
    // Regex: ^(?:n1|n2|e2|t2d|c2|c2d|m1|m2)-(?:standard|highcpu|highmem)-(?:\d+)$
    let s = s.trim();
    let mut parts = s.split('-');
    let fam = parts.next().unwrap_or("");
    let kind = parts.next().unwrap_or("");
    let n = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return false;
    }
    matches!(fam, "n1" | "n2" | "e2" | "t2d" | "c2" | "c2d" | "m1" | "m2")
        && matches!(kind, "standard" | "highcpu" | "highmem")
        && !n.is_empty()
        && n.chars().all(|c| c.is_ascii_digit())
}

fn is_azure_instance_type(s: &str) -> bool {
    // Regex: ^Standard_[A-Za-z0-9]+$
    let s = s.trim();
    let Some(rest) = s.strip_prefix("Standard_") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric())
}

fn is_number(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for c in s.chars() {
        if c.is_ascii_digit() {
            seen_digit = true;
            continue;
        }
        if c == '.' && !seen_dot {
            seen_dot = true;
            continue;
        }
        return false;
    }
    seen_digit
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_diagnostics::SourceLocation;
    use sruja_language::{
        ElementAssignment, ElementDef, ElementDefBody, ElementKind, MetaEntry, Program,
        TopLevelItem,
    };

    fn program_with_metadata(entries: Vec<MetaEntry>) -> Program {
        let loc = SourceLocation::new(String::new(), 0, 0);
        let body = ElementDefBody {
            metadata: entries,
            ..Default::default()
        };
        let elem = ElementDef {
            location: loc.clone(),
            assignment: ElementAssignment {
                location: loc.clone(),
                name: "api".to_string(),
                kind: ElementKind::Container,
                sub_kind: None,
                title: Some("API".to_string()),
                tag_refs: vec![],
                body: Some(body),
            },
        };
        Program {
            items: vec![TopLevelItem::ElementDef(Box::new(elem))],
        }
    }

    #[test]
    fn empty_program_returns_no_diagnostics() {
        let program = Program { items: vec![] };
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_read_replicas_passes() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "capacity.readReplicas".to_string(),
            value: Some("3".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_read_replicas_fails() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "capacity.readReplicas".to_string(),
            value: Some("three".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("capacity.readReplicas"));
    }

    #[test]
    fn valid_percentage_obs_tracing_passes() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "obs.tracing.sampleRate".to_string(),
            value: Some("10%".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_percentage_obs_tracing_fails() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "obs.tracing.sampleRate".to_string(),
            value: Some("ten".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn valid_compliance_pci_passes() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "compliance.pci.level".to_string(),
            value: Some("1".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_compliance_pci_fails() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "compliance.pci.level".to_string(),
            value: Some("".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn valid_currency_cost_passes() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "cost.monthly.total".to_string(),
            value: Some("$1,000.50".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_currency_cost_fails() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "cost.monthly.total".to_string(),
            value: Some("1000".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn valid_aws_instance_type_passes() {
        let program = program_with_metadata(vec![
            MetaEntry {
                key: "capacity.instanceProvider".to_string(),
                value: Some("aws".to_string()),
            },
            MetaEntry {
                key: "capacity.instanceType".to_string(),
                value: Some("t3.medium".to_string()),
            },
        ]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_gcp_instance_type_passes() {
        let program = program_with_metadata(vec![
            MetaEntry {
                key: "capacity.instanceProvider".to_string(),
                value: Some("gcp".to_string()),
            },
            MetaEntry {
                key: "capacity.instanceType".to_string(),
                value: Some("n1-standard-4".to_string()),
            },
        ]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn valid_azure_instance_type_passes() {
        let program = program_with_metadata(vec![
            MetaEntry {
                key: "capacity.instanceProvider".to_string(),
                value: Some("azure".to_string()),
            },
            MetaEntry {
                key: "capacity.instanceType".to_string(),
                value: Some("Standard_D4s".to_string()),
            },
        ]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty());
    }

    #[test]
    fn invalid_aws_instance_type_fails() {
        let program = program_with_metadata(vec![
            MetaEntry {
                key: "capacity.instanceProvider".to_string(),
                value: Some("aws".to_string()),
            },
            MetaEntry {
                key: "capacity.instanceType".to_string(),
                value: Some("invalid-type".to_string()),
            },
        ]);
        let diags = PropertiesValidationRule.validate(&program);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn unknown_property_ignored() {
        let program = program_with_metadata(vec![MetaEntry {
            key: "customKey".to_string(),
            value: Some("anything".to_string()),
        }]);
        let diags = PropertiesValidationRule.validate(&program);
        assert!(diags.is_empty(), "unknown properties are not validated");
    }
}

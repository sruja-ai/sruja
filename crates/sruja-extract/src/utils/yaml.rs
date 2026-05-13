//! YAML parsing utilities for extractors.

use std::collections::HashSet;

pub fn extract_title_from_yaml(content: &str, prefixes: &[&str]) -> Option<String> {
    let mut in_info_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "info:" {
            in_info_block = true;
            continue;
        }

        for prefix in prefixes {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let is_title_prefix =
                    prefix == &"title:" || prefix == &"\"title\":" || prefix == &"'title':";
                if in_info_block || is_title_prefix {
                    let title = rest
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .trim_matches(',')
                        .to_string();
                    if !title.is_empty() {
                        return Some(title);
                    }
                }
            }
        }

        if in_info_block && !trimmed.is_empty() {
            let indent = line.len() - line.trim_start().len();
            if indent == 0 && trimmed != "info:" {
                in_info_block = false;
            }
        }
    }

    for line in content.lines().take(20) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("title:") {
            let title = rest
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim_matches(',')
                .to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
        if let Some(rest) = trimmed.strip_prefix("\"title\":") {
            let title = rest.trim().trim_matches('"').trim_matches(',').to_string();
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    None
}

pub fn has_markers(content: &str, markers: &[&str]) -> bool {
    markers.iter().any(|m| content.contains(m))
}

pub fn parse_yaml_services(content: &str) -> Vec<String> {
    let mut services = Vec::new();
    let mut in_services = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if trimmed.starts_with("services:") && indent == 0 {
            in_services = true;
            continue;
        }

        if in_services {
            if indent == 0 && !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }

            if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('#') {
                let service_name = trimmed.trim_end_matches(':').trim().to_string();
                if !service_name.is_empty() {
                    services.push(service_name);
                }
            }
        }
    }

    services
}

pub fn parse_yaml_resources(content: &str, valid_kinds: &[&str]) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current_kind: Option<String> = None;
    let mut current_name: Option<String> = None;
    let mut in_metadata = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "---" {
            if let (Some(kind), Some(name)) = (current_kind.take(), current_name.take()) {
                results.push((kind, name));
            }
            in_metadata = false;
            continue;
        }

        if let Some(kind_str) = trimmed.strip_prefix("kind:") {
            let kind = kind_str.trim().trim_matches('"').to_string();
            if valid_kinds.contains(&kind.as_str()) {
                current_kind = Some(kind);
            }
        } else if trimmed == "metadata:" {
            in_metadata = true;
        } else if let Some(rest) = trimmed.strip_prefix("name:") {
            if in_metadata {
                let indent = line.len() - line.trim_start().len();
                if indent <= 4 {
                    current_name = Some(rest.trim().trim_matches('"').to_string());
                    in_metadata = false;
                }
            }
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            let indent = line.len() - line.trim_start().len();
            if indent == 0 && trimmed != "apiVersion:" && !trimmed.starts_with("apiVersion:") {
                in_metadata = false;
            }
        }
    }

    if let (Some(kind), Some(name)) = (current_kind, current_name) {
        results.push((kind, name));
    }

    results
}

pub fn extract_key_value_pairs(content: &str, keys: &[&str]) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let key_set: HashSet<&str> = keys.iter().copied().collect();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim();
            let value = trimmed[eq_pos + 1..].trim();
            if key_set.contains(key) && !value.is_empty() {
                let service_name = normalize_service_name(key);
                if !service_name.is_empty() {
                    results.push((service_name, value.to_string()));
                }
            }
        }

        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();
            if key_set.contains(key) && !value.is_empty() {
                let service_name = normalize_service_name(key);
                if !service_name.is_empty() {
                    results.push((service_name, value.to_string()));
                }
            }
        }
    }

    deduplicate_by_key(&results)
}

fn normalize_service_name(key: &str) -> String {
    let key = key.trim();
    let normalized = key
        .to_uppercase()
        .replace("_HOST", "")
        .replace("_URL", "")
        .replace("_SERVICE_URL", "")
        .replace("_SERVICE_HOST", "")
        .replace("_BASE_URL", "");

    if normalized.is_empty() || normalized == key {
        return String::new();
    }

    normalized
        .split('_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect::<Vec<_>>()
        .join("-")
}

fn deduplicate_by_key(items: &[(String, String)]) -> Vec<(String, String)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut results = Vec::new();

    for item in items {
        if seen.insert(item.0.clone()) {
            results.push(item.clone());
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yaml_services_basic() {
        let content = "services:\n  api:\n    image: api:v1\n  worker:\n    image: worker:v1";
        let services = parse_yaml_services(content);
        assert_eq!(services, vec!["api", "worker"]);
    }

    #[test]
    fn test_parse_yaml_services_empty() {
        let content = "version: 3\nnetworks:\n  api:";
        let services = parse_yaml_services(content);
        assert!(services.is_empty());
    }

    #[test]
    fn test_parse_yaml_services_with_comments() {
        let content = "services:\n  # comment\n  api:\n    image: api:v1";
        let services = parse_yaml_services(content);
        assert_eq!(services, vec!["api"]);
    }

    #[test]
    fn test_parse_yaml_resources_basic() {
        let content = "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: api\n---\napiVersion: v1\nkind: Service\nmetadata:\n  name: api-svc";
        let kinds = &["Deployment", "Service"];
        let resources = parse_yaml_resources(content, kinds);
        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0], ("Deployment".to_string(), "api".to_string()));
        assert_eq!(resources[1], ("Service".to_string(), "api-svc".to_string()));
    }

    #[test]
    fn test_extract_key_value_pairs() {
        let content = "PAYMENT_SERVICE_URL=https://api.example.com\nUSER_HOST=localhost:8080";
        let keys = &["PAYMENT_SERVICE_URL", "USER_HOST"];
        let pairs = extract_key_value_pairs(content, keys);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].0, "payment-service");
        assert_eq!(pairs[0].1, "https://api.example.com");
    }

    #[test]
    fn test_deduplicate_by_key() {
        let items = vec![
            ("api".to_string(), "http://api1".to_string()),
            ("api".to_string(), "http://api2".to_string()),
            ("user".to_string(), "http://user".to_string()),
        ];
        let result = deduplicate_by_key(&items);
        assert_eq!(result.len(), 2);
    }
}

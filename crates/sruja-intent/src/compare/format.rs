use sruja_scan::Node;

use crate::model::PolicySelector;

pub fn node_matches_selector(node: &Node, selector: &PolicySelector) -> bool {
    if let Some(ref kind) = selector.kind {
        let normalize_kind = |s: &str| s.replace([' ', '-'], "_").to_lowercase();
        if normalize_kind(node.kind.as_str()) != normalize_kind(kind) {
            return false;
        }
    }
    if let Some(ref id) = selector.id {
        if node.id != *id && !node.id.contains(id) {
            return false;
        }
    }
    if let Some(ref tech) = selector.technology {
        if node
            .technology
            .as_ref()
            .map(|t| t.to_lowercase() != tech.to_lowercase())
            .unwrap_or(true)
        {
            return false;
        }
    }
    for tag in &selector.tags {
        let has_tag = node.metadata.contains_key(tag)
            || node
                .metadata
                .get("tags")
                .map(|t: &String| t.split(',').any(|s| s.trim() == tag))
                .unwrap_or(false);
        if !has_tag {
            return false;
        }
    }
    for meta in &selector.meta {
        if let Some(ref val) = meta.value {
            if node.metadata.get(&meta.key) != Some(val) {
                return false;
            }
        } else if !node.metadata.contains_key(&meta.key) {
            return false;
        }
    }
    true
}

pub fn node_matches_selector_strict(node: &Node, selector: &PolicySelector) -> bool {
    if let Some(ref kind) = selector.kind {
        let normalize_kind = |s: &str| s.replace([' ', '-'], "_").to_lowercase();
        if normalize_kind(node.kind.as_str()) != normalize_kind(kind) {
            return false;
        }
    }
    if let Some(ref id) = selector.id {
        if node.id != *id {
            return false;
        }
    }
    if let Some(ref tech) = selector.technology {
        if node
            .technology
            .as_ref()
            .map(|t| t.to_lowercase() != tech.to_lowercase())
            .unwrap_or(true)
        {
            return false;
        }
    }
    for tag in &selector.tags {
        let has_tag = node.metadata.contains_key(tag)
            || node
                .metadata
                .get("tags")
                .map(|t: &String| t.split(',').any(|s| s.trim() == tag))
                .unwrap_or(false);
        if !has_tag {
            return false;
        }
    }
    for meta in &selector.meta {
        if let Some(ref val) = meta.value {
            if node.metadata.get(&meta.key) != Some(val) {
                return false;
            }
        } else if !node.metadata.contains_key(&meta.key) {
            return false;
        }
    }
    true
}

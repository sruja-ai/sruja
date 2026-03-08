//! Question-based coverage evaluation (heuristic).
//!
//! Evaluates if selected components can answer key architecture questions.

use sruja_scan::{Graph, Node};
use std::collections::HashSet;

pub const ARCHITECTURE_QUESTIONS: &[&str] = &[
    "What are the main services and their responsibilities?",
    "How is data stored and accessed?",
    "What external systems does this integrate with?",
    "What are the main API endpoints?",
    "How is authentication/authorization handled?",
    "What are the core domain concepts?",
    "How is configuration managed?",
    "What are the main entry points to the system?",
    "How do components communicate with each other?",
    "What are the critical data flows?",
];

#[derive(Debug, Clone)]
pub struct QuestionCoverageResult {
    pub question: String,
    pub score: f64,
    pub relevant_components: Vec<String>,
}

pub async fn evaluate_question_coverage(
    selection: &[Node],
    graph: &Graph,
    _llm_enabled: bool,
) -> Vec<QuestionCoverageResult> {
    let mut results = Vec::new();

    if selection.is_empty() {
        return ARCHITECTURE_QUESTIONS
            .iter()
            .map(|q| QuestionCoverageResult {
                question: q.to_string(),
                score: 0.0,
                relevant_components: vec![],
            })
            .collect();
    }

    let selected_ids: HashSet<_> = selection.iter().map(|n| n.id.as_str()).collect();

    for question in ARCHITECTURE_QUESTIONS {
        let score = evaluate_heuristically(question, selection, graph, &selected_ids);

        let relevant = find_relevant_components(question, selection);

        results.push(QuestionCoverageResult {
            question: question.to_string(),
            score,
            relevant_components: relevant,
        });
    }

    results
}

pub async fn refine_for_questions(
    graph: &Graph,
    initial_selection: &[Node],
    llm_enabled: bool,
    target_score: f64,
    max_iterations: usize,
) -> Vec<Node> {
    let mut selection = initial_selection.to_vec();
    let mut selected_ids: HashSet<String> = selection.iter().map(|n| n.id.clone()).collect();

    for iteration in 0..max_iterations {
        let results = evaluate_question_coverage(&selection, graph, llm_enabled).await;

        let avg_score = results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64;

        if avg_score >= target_score {
            eprintln!("   ✓ Target coverage achieved: {:.0}%", avg_score * 100.0);
            break;
        }

        let weak_questions: Vec<_> = results.iter().filter(|r| r.score < target_score).collect();

        if weak_questions.is_empty() {
            break;
        }

        let candidates: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| !selected_ids.contains(&n.id))
            .collect();

        if candidates.is_empty() {
            break;
        }

        let best_candidate = find_best_addition(&weak_questions, candidates, graph);

        if let Some(node) = best_candidate {
            selected_ids.insert(node.id.clone());
            selection.push(node.clone());

            if iteration % 5 == 0 {
                eprintln!(
                    "   Iteration {}: {} components, score {:.0}%",
                    iteration + 1,
                    selection.len(),
                    avg_score * 100.0
                );
            }
        } else {
            break;
        }
    }

    selection
}

fn evaluate_heuristically(
    question: &str,
    selection: &[Node],
    graph: &sruja_scan::Graph,
    selected_ids: &HashSet<&str>,
) -> f64 {
    let question_lower = question.to_lowercase();

    let relevant_count = selection
        .iter()
        .filter(|n| {
            let path = n.path.as_deref().unwrap_or(&n.label).to_lowercase();

            if question_lower.contains("service") && path.contains("service") {
                return true;
            }
            if question_lower.contains("api")
                && (path.contains("api") || path.contains("controller") || path.contains("handler"))
            {
                return true;
            }
            if question_lower.contains("data")
                && (path.contains("repo") || path.contains("db") || path.contains("database"))
            {
                return true;
            }
            if question_lower.contains("auth")
                && (path.contains("auth") || path.contains("security"))
            {
                return true;
            }
            if question_lower.contains("external")
                && (path.contains("client") || path.contains("integration"))
            {
                return true;
            }
            if question_lower.contains("config") && path.contains("config") {
                return true;
            }
            if question_lower.contains("entry")
                && (path.contains("main") || path.contains("index") || path.contains("app"))
            {
                return true;
            }

            false
        })
        .count();

    let total_relevant_in_graph = graph
        .nodes
        .iter()
        .filter(|n| selected_ids.contains(n.id.as_str()))
        .count()
        .max(1);

    (relevant_count as f64 / total_relevant_in_graph as f64).min(1.0)
}

fn find_relevant_components(question: &str, selection: &[Node]) -> Vec<String> {
    let question_lower = question.to_lowercase();

    selection
        .iter()
        .filter_map(|n| {
            let path = n.path.as_deref().unwrap_or(&n.label);
            let path_lower = path.to_lowercase();

            let is_relevant = (question_lower.contains("service")
                && path_lower.contains("service"))
                || (question_lower.contains("api")
                    && (path_lower.contains("api") || path_lower.contains("controller")))
                || (question_lower.contains("data")
                    && (path_lower.contains("repo") || path_lower.contains("db")))
                || (question_lower.contains("auth") && path_lower.contains("auth"))
                || (question_lower.contains("config") && path_lower.contains("config"));

            if is_relevant {
                Some(path.to_string())
            } else {
                None
            }
        })
        .take(10)
        .collect()
}

fn find_best_addition<'a>(
    weak_questions: &[&QuestionCoverageResult],
    candidates: Vec<&'a Node>,
    _graph: &Graph,
) -> Option<&'a Node> {
    let keywords: Vec<String> = weak_questions
        .iter()
        .flat_map(|r| {
            let q = r.question.to_lowercase();
            let mut kws = Vec::new();
            if q.contains("service") {
                kws.push("service".to_string());
            }
            if q.contains("api") {
                kws.push("api".to_string());
                kws.push("controller".to_string());
            }
            if q.contains("data") {
                kws.push("repo".to_string());
                kws.push("db".to_string());
            }
            if q.contains("auth") {
                kws.push("auth".to_string());
            }
            if q.contains("config") {
                kws.push("config".to_string());
            }
            if q.contains("external") {
                kws.push("client".to_string());
            }
            kws
        })
        .collect();

    candidates.into_iter().max_by(|a, b| {
        let score_a = score_for_keywords(a.path.as_deref().unwrap_or(&a.label), &keywords);
        let score_b = score_for_keywords(b.path.as_deref().unwrap_or(&b.label), &keywords);
        score_a.cmp(&score_b)
    })
}

fn score_for_keywords(text: &str, keywords: &[String]) -> usize {
    let text_lower = text.to_lowercase();
    keywords
        .iter()
        .filter(|kw| text_lower.contains(&kw.to_lowercase()))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sruja_scan::NodeKind;

    #[tokio::test]
    async fn test_evaluate_empty_selection() {
        let results = evaluate_question_coverage(&[], &Graph::default(), false).await;
        assert_eq!(results.len(), ARCHITECTURE_QUESTIONS.len());
        assert!(results.iter().all(|r| r.score == 0.0));
    }

    #[test]
    fn test_find_relevant_components() {
        let nodes = vec![
            Node {
                id: "1".into(),
                label: "UserService".into(),
                path: Some("src/services/user.rs".into()),
                kind: NodeKind::Module,
                technology: None,
                metadata: Default::default(),
            },
            Node {
                id: "2".into(),
                label: "Config".into(),
                path: Some("src/config.rs".into()),
                kind: NodeKind::Module,
                technology: None,
                metadata: Default::default(),
            },
        ];

        let relevant = find_relevant_components("What are the main services?", &nodes);
        assert!(relevant.iter().any(|r| r.contains("user")));
    }
}

//! Coupling Metrics for Architecture Analysis
//!
//! Implements Martin's metrics for package/module coupling:
//! - Afferent coupling (Ca): incoming dependencies
//! - Efferent coupling (Ce): outgoing dependencies
//! - Instability: Ce / (Ca + Ce)
//! - Abstractness: ratio of abstract types
//! - Distance from main sequence: |A + I - 1|

use std::collections::HashMap;

pub struct CouplingAnalyzer;

pub struct CouplingResult {
    pub afferent: HashMap<String, usize>,
    pub efferent: HashMap<String, usize>,
    pub instability: HashMap<String, f64>,
    pub abstractness: HashMap<String, f64>,
    pub distance: HashMap<String, f64>,
    pub violations: Vec<CouplingViolation>,
    pub summary: CouplingSummary,
}

pub struct CouplingViolation {
    pub module: String,
    pub violation_type: CouplingViolationType,
    pub current_instability: f64,
    pub current_abstractness: f64,
    pub current_distance: f64,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouplingViolationType {
    ZoneOfPain,
    ZoneOfUselessness,
    OverlyStable,
    OverlyUnstable,
}

pub struct CouplingSummary {
    pub total_modules: usize,
    pub avg_instability: f64,
    pub avg_abstractness: f64,
    pub avg_distance: f64,
    pub pain_zone_count: usize,
    pub uselessness_zone_count: usize,
    pub healthy_count: usize,
}

pub struct ModuleCoupling {
    pub module: String,
    pub ca: usize,
    pub ce: usize,
    pub instability: f64,
    pub abstractness: f64,
    pub distance: f64,
    pub zone: Zone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    MainSequence,
    ZoneOfPain,
    ZoneOfUselessness,
}

impl Default for CouplingAnalyzer {
    fn default() -> Self {
        Self
    }
}

impl CouplingAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, nodes: &[String], edges: &[(String, String)]) -> CouplingResult {
        self.analyze_with_abstractness(nodes, edges, &HashMap::new())
    }

    pub fn analyze_with_abstractness(
        &self,
        nodes: &[String],
        edges: &[(String, String)],
        abstract_counts: &HashMap<String, usize>,
    ) -> CouplingResult {
        let (afferent, efferent) = self.compute_coupling(nodes, edges);
        let instability = self.compute_instability(&afferent, &efferent);
        let abstractness = self.compute_abstractness(nodes, abstract_counts);
        let distance = self.compute_distance(&instability, &abstractness);
        let violations = self.detect_violations(&instability, &abstractness, &distance);
        let summary = self.compute_summary(&instability, &abstractness, &distance, &violations);

        CouplingResult {
            afferent,
            efferent,
            instability,
            abstractness,
            distance,
            violations,
            summary,
        }
    }

    pub fn analyze_modules(
        &self,
        nodes: &[String],
        edges: &[(String, String)],
    ) -> Vec<ModuleCoupling> {
        let (afferent, efferent) = self.compute_coupling(nodes, edges);
        let instability = self.compute_instability(&afferent, &efferent);
        let abstractness = self.compute_abstractness(nodes, &HashMap::new());
        let distance = self.compute_distance(&instability, &abstractness);

        nodes
            .iter()
            .map(|node| {
                let ca = afferent.get(node).copied().unwrap_or(0);
                let ce = efferent.get(node).copied().unwrap_or(0);
                let i = instability.get(node).copied().unwrap_or(0.0);
                let a = abstractness.get(node).copied().unwrap_or(0.0);
                let d = distance.get(node).copied().unwrap_or(0.0);

                let zone = self.classify_zone(i, a);

                ModuleCoupling {
                    module: node.clone(),
                    ca,
                    ce,
                    instability: i,
                    abstractness: a,
                    distance: d,
                    zone,
                }
            })
            .collect()
    }

    fn compute_coupling(
        &self,
        nodes: &[String],
        edges: &[(String, String)],
    ) -> (HashMap<String, usize>, HashMap<String, usize>) {
        let mut afferent: HashMap<String, usize> = HashMap::new();
        let mut efferent: HashMap<String, usize> = HashMap::new();

        for node in nodes {
            afferent.insert(node.clone(), 0);
            efferent.insert(node.clone(), 0);
        }

        for (source, target) in edges {
            *efferent.entry(source.clone()).or_default() += 1;
            *afferent.entry(target.clone()).or_default() += 1;
        }

        (afferent, efferent)
    }

    fn compute_instability(
        &self,
        afferent: &HashMap<String, usize>,
        efferent: &HashMap<String, usize>,
    ) -> HashMap<String, f64> {
        afferent
            .keys()
            .map(|node| {
                let ca = afferent.get(node).copied().unwrap_or(0);
                let ce = efferent.get(node).copied().unwrap_or(0);
                let total = ca + ce;

                let instability = if total == 0 {
                    0.0
                } else {
                    ce as f64 / total as f64
                };

                (node.clone(), instability)
            })
            .collect()
    }

    fn compute_abstractness(
        &self,
        nodes: &[String],
        abstract_counts: &HashMap<String, usize>,
    ) -> HashMap<String, f64> {
        nodes
            .iter()
            .map(|node| {
                let abstract_count = abstract_counts.get(node).copied().unwrap_or(0);
                let total = 1usize.max(abstract_count);
                let abstractness = abstract_count as f64 / total as f64;
                (node.clone(), abstractness.min(1.0))
            })
            .collect()
    }

    fn compute_distance(
        &self,
        instability: &HashMap<String, f64>,
        abstractness: &HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        instability
            .keys()
            .map(|node| {
                let i = instability.get(node).copied().unwrap_or(0.0);
                let a = abstractness.get(node).copied().unwrap_or(0.0);
                let distance = (a + i - 1.0).abs();
                (node.clone(), distance)
            })
            .collect()
    }

    fn classify_zone(&self, instability: f64, abstractness: f64) -> Zone {
        let distance = (abstractness + instability - 1.0).abs();

        if distance <= 0.3 {
            Zone::MainSequence
        } else if instability < 0.3 && abstractness < 0.3 {
            Zone::ZoneOfPain
        } else if instability > 0.7 && abstractness > 0.7 {
            Zone::ZoneOfUselessness
        } else {
            Zone::MainSequence
        }
    }

    fn detect_violations(
        &self,
        instability: &HashMap<String, f64>,
        abstractness: &HashMap<String, f64>,
        distance: &HashMap<String, f64>,
    ) -> Vec<CouplingViolation> {
        instability
            .keys()
            .filter_map(|module| {
                let i = instability.get(module).copied().unwrap_or(0.0);
                let a = abstractness.get(module).copied().unwrap_or(0.0);
                let d = distance.get(module).copied().unwrap_or(0.0);

                let (violation_type, suggestion) = if d > 0.7 && i < 0.3 && a < 0.3 {
                    (
                        CouplingViolationType::ZoneOfPain,
                        format!(
                            "Module '{}' is in the Zone of Pain: concrete + stable. \
                             Consider making it more abstract or reducing its dependents.",
                            module
                        ),
                    )
                } else if d > 0.7 && i > 0.7 && a > 0.7 {
                    (
                        CouplingViolationType::ZoneOfUselessness,
                        format!(
                            "Module '{}' is in the Zone of Uselessness: abstract + unstable. \
                             Consider making it more concrete or adding dependents.",
                            module
                        ),
                    )
                } else if i < 0.1 && a < 0.5 {
                    (
                        CouplingViolationType::OverlyStable,
                        format!(
                            "Module '{}' is overly stable but not abstract enough. \
                             Consider adding abstractions or interfaces.",
                            module
                        ),
                    )
                } else if i > 0.9 && a > 0.5 {
                    (
                        CouplingViolationType::OverlyUnstable,
                        format!(
                            "Module '{}' is highly unstable but has abstractions. \
                             Consider making it more concrete or reducing its dependencies.",
                            module
                        ),
                    )
                } else {
                    return None;
                };

                Some(CouplingViolation {
                    module: module.clone(),
                    violation_type,
                    current_instability: i,
                    current_abstractness: a,
                    current_distance: d,
                    suggestion,
                })
            })
            .collect()
    }

    fn compute_summary(
        &self,
        instability: &HashMap<String, f64>,
        abstractness: &HashMap<String, f64>,
        distance: &HashMap<String, f64>,
        violations: &[CouplingViolation],
    ) -> CouplingSummary {
        let total = instability.len();

        let avg_instability = if total > 0 {
            instability.values().sum::<f64>() / total as f64
        } else {
            0.0
        };

        let avg_abstractness = if total > 0 {
            abstractness.values().sum::<f64>() / total as f64
        } else {
            0.0
        };

        let avg_distance = if total > 0 {
            distance.values().sum::<f64>() / total as f64
        } else {
            0.0
        };

        let pain_zone_count = violations
            .iter()
            .filter(|v| v.violation_type == CouplingViolationType::ZoneOfPain)
            .count();

        let uselessness_zone_count = violations
            .iter()
            .filter(|v| v.violation_type == CouplingViolationType::ZoneOfUselessness)
            .count();

        let healthy_count = total - violations.len();

        CouplingSummary {
            total_modules: total,
            avg_instability,
            avg_abstractness,
            avg_distance,
            pain_zone_count,
            uselessness_zone_count,
            healthy_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let analyzer = CouplingAnalyzer::new();
        let result = analyzer.analyze(&[], &[]);
        assert!(result.afferent.is_empty());
        assert!(result.summary.total_modules == 0);
    }

    #[test]
    fn test_single_node() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let result = analyzer.analyze(&nodes, &[]);

        assert_eq!(result.afferent["a"], 0);
        assert_eq!(result.efferent["a"], 0);
        assert_eq!(result.instability["a"], 0.0);
    }

    #[test]
    fn test_two_nodes_connected() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![("a".to_string(), "b".to_string())];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.efferent["a"], 1);
        assert_eq!(result.afferent["b"], 1);
        assert_eq!(result.instability["a"], 1.0);
        assert_eq!(result.instability["b"], 0.0);
    }

    #[test]
    fn test_instability_calculation() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("a".to_string(), "c".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.instability["a"], 1.0);
        assert!((result.instability["b"] - 0.5).abs() < 0.01);
        assert_eq!(result.instability["c"], 0.0);
    }

    #[test]
    fn test_distance_from_main_sequence() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let result = analyzer.analyze(&nodes, &[]);

        assert!((result.distance["a"] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_module_coupling_analysis() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![("a".to_string(), "b".to_string())];
        let modules = analyzer.analyze_modules(&nodes, &edges);

        assert_eq!(modules.len(), 2);

        let module_a = modules.iter().find(|m| m.module == "a").unwrap();
        assert_eq!(module_a.ce, 1);
        assert_eq!(module_a.ca, 0);
        assert_eq!(module_a.zone, Zone::MainSequence);
    }

    #[test]
    fn test_violation_detection() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec![
            "stable_concrete".to_string(),
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ];
        let edges = vec![
            ("a".to_string(), "stable_concrete".to_string()),
            ("b".to_string(), "stable_concrete".to_string()),
            ("c".to_string(), "stable_concrete".to_string()),
        ];
        let result = analyzer.analyze(&nodes, &edges);

        let stable_violation = result
            .violations
            .iter()
            .find(|v| v.module == "stable_concrete");

        assert!(stable_violation.is_some());
        assert_eq!(
            stable_violation.unwrap().violation_type,
            CouplingViolationType::ZoneOfPain
        );
    }

    #[test]
    fn test_summary_calculation() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string(), "b".to_string()];
        let edges = vec![("a".to_string(), "b".to_string())];
        let result = analyzer.analyze(&nodes, &edges);

        assert_eq!(result.summary.total_modules, 2);
        assert!(result.summary.avg_instability >= 0.0);
    }

    #[test]
    fn test_abstractness_with_counts() {
        let analyzer = CouplingAnalyzer::new();
        let nodes = vec!["a".to_string()];
        let mut abstract_counts = HashMap::new();
        abstract_counts.insert("a".to_string(), 1);

        let result = analyzer.analyze_with_abstractness(&nodes, &[], &abstract_counts);

        assert!((result.abstractness["a"] - 1.0).abs() < 0.01);
    }
}

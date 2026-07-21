use std::collections::{HashMap, HashSet};

use super::types::FanoutList;

pub(crate) fn pagerank(graph: &HashMap<String, Vec<String>>) -> HashMap<String, f64> {
    if graph.is_empty() {
        return HashMap::new();
    }

    let damping = 0.85;
    let iterations = 20;

    let mut nodes: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (source, targets) in graph {
        nodes.insert(source.clone());
        for target in targets {
            nodes.insert(target.clone());
        }
    }
    let nodes: Vec<String> = nodes.into_iter().collect();
    let n = nodes.len().max(1);

    let mut scores: HashMap<String, f64> =
        nodes.iter().map(|k| (k.clone(), 1.0 / n as f64)).collect();

    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
    for (source, targets) in graph {
        for target in targets {
            incoming
                .entry(target.clone())
                .or_default()
                .push(source.clone());
        }
    }
    for preds in incoming.values_mut() {
        preds.sort();
        preds.dedup();
    }

    for _ in 0..iterations {
        let mut new_scores: HashMap<String, f64> = HashMap::new();
        for node in &nodes {
            let mut score = (1.0 - damping) / n as f64;

            if let Some(predecessors) = incoming.get(node) {
                for pred in predecessors {
                    let pred_score = scores.get(pred).copied().unwrap_or(0.0);
                    let out_degree = graph
                        .get(pred)
                        .map(|v: &Vec<String>| v.len().max(1) as f64)
                        .unwrap_or(1.0);
                    score += damping * pred_score / out_degree;
                }
            }

            new_scores.insert(node.clone(), score);
        }

        scores = new_scores;
    }

    scores
}

pub(crate) fn fan_in_out(
    import_graph: &HashMap<String, Vec<String>>,
) -> (FanoutList, FanoutList) {
    let mut out_counts: HashMap<&str, usize> = HashMap::new();
    let mut in_counts: HashMap<&str, usize> = HashMap::new();
    for (src, targets) in import_graph {
        out_counts.insert(src.as_str(), targets.len());
        for tgt in targets {
            *in_counts.entry(tgt.as_str()).or_insert(0) += 1;
        }
    }
    let mut fan_out: Vec<(String, usize)> = out_counts
        .into_iter()
        .filter(|(_, c)| *c >= 15)
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    fan_out.sort_by_key(|item| std::cmp::Reverse(item.1));

    let mut fan_in: Vec<(String, usize)> = in_counts
        .into_iter()
        .filter(|(_, c)| *c >= 15)
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    fan_in.sort_by_key(|item| std::cmp::Reverse(item.1));

    (fan_out, fan_in)
}

pub(crate) fn format_fanout(label: &str, items: &[(String, usize)]) -> Option<String> {
    let top: Vec<String> = items
        .iter()
        .take(3)
        .map(|(p, c)| format!("{} ({})", p, c))
        .collect();
    if top.is_empty() {
        None
    } else {
        Some(format!("- {} modules (top: {}).", label, top.join(", ")))
    }
}

pub(crate) fn find_dependency_cycles(
    graph: &HashMap<String, Vec<String>>,
    max_cycles: usize,
) -> Vec<Vec<String>> {
    struct Tarjan<'a> {
        graph: &'a HashMap<String, Vec<String>>,
        index: usize,
        stack: Vec<String>,
        on_stack: HashSet<String>,
        indices: HashMap<String, usize>,
        lowlink: HashMap<String, usize>,
        sccs: Vec<Vec<String>>,
    }

    impl<'a> Tarjan<'a> {
        fn new(graph: &'a HashMap<String, Vec<String>>) -> Self {
            Self {
                graph,
                index: 0,
                stack: Vec::new(),
                on_stack: HashSet::new(),
                indices: HashMap::new(),
                lowlink: HashMap::new(),
                sccs: Vec::new(),
            }
        }

        fn strongconnect(&mut self, v: String) {
            self.indices.insert(v.clone(), self.index);
            self.lowlink.insert(v.clone(), self.index);
            self.index += 1;
            self.stack.push(v.clone());
            self.on_stack.insert(v.clone());

            if let Some(targets) = self.graph.get(&v) {
                for w in targets {
                    if !self.indices.contains_key(w) {
                        self.strongconnect(w.clone());
                        let v_low = *self.lowlink.get(&v).unwrap_or(&0);
                        let w_low = *self.lowlink.get(w).unwrap_or(&0);
                        self.lowlink.insert(v.clone(), v_low.min(w_low));
                    } else if self.on_stack.contains(w) {
                        let v_low = *self.lowlink.get(&v).unwrap_or(&0);
                        let w_idx = *self.indices.get(w).unwrap_or(&0);
                        self.lowlink.insert(v.clone(), v_low.min(w_idx));
                    }
                }
            }

            let v_idx = *self.indices.get(&v).unwrap_or(&0);
            let v_low = *self.lowlink.get(&v).unwrap_or(&0);
            if v_low == v_idx {
                let mut scc: Vec<String> = Vec::new();
                while let Some(w) = self.stack.pop() {
                    self.on_stack.remove(&w);
                    scc.push(w.clone());
                    if w == v {
                        break;
                    }
                }
                if scc.len() >= 2 {
                    self.sccs.push(scc);
                }
            }
        }
    }

    let mut nodes: HashSet<String> = HashSet::new();
    for (src, targets) in graph {
        nodes.insert(src.clone());
        for t in targets {
            nodes.insert(t.clone());
        }
    }

    let mut nodes_vec: Vec<String> = nodes.into_iter().collect();
    nodes_vec.sort();
    let mut tarjan = Tarjan::new(graph);
    for v in nodes_vec {
        if !tarjan.indices.contains_key(&v) {
            tarjan.strongconnect(v);
        }
    }

    let mut sccs = tarjan.sccs;
    sccs.sort_by_key(|scc| std::cmp::Reverse(scc.len()));
    sccs.truncate(max_cycles);
    sccs
}

use sruja_scan::Graph;
use std::collections::{HashMap, HashSet};

/// Find circular dependencies in the graph using Tarjan's SCC algorithm.
/// Returns deduplicated cycles (canonicalized by lexicographically smallest rotation).
pub fn find_circular_dependencies(graph: &Graph) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.source.as_str())
            .or_default()
            .push(edge.target.as_str());
    }

    let sccs = tarjan_scc(&graph.nodes, &adj);

    let mut result = Vec::new();
    for scc in sccs {
        if scc.len() > 1 {
            if let Some(cycle) = find_cycle_in_scc(&scc, &adj) {
                let canonical = canonicalize_cycle(&cycle);
                result.push(canonical);
            }
        }
    }

    result
}

/// Tarjan's algorithm for finding strongly connected components
fn tarjan_scc<'a>(
    nodes: &'a [sruja_scan::Node],
    adj: &HashMap<&'a str, Vec<&'a str>>,
) -> Vec<Vec<&'a str>> {
    let mut index_counter = 0usize;
    let mut stack: Vec<&'a str> = Vec::new();
    let mut indices: HashMap<&'a str, usize> = HashMap::new();
    let mut lowlinks: HashMap<&'a str, usize> = HashMap::new();
    let mut on_stack: HashSet<&'a str> = HashSet::new();
    let mut sccs: Vec<Vec<&'a str>> = Vec::new();

    for node in nodes {
        if !indices.contains_key(node.id.as_str()) {
            tarjan_strongconnect(
                node.id.as_str(),
                adj,
                &mut index_counter,
                &mut stack,
                &mut indices,
                &mut lowlinks,
                &mut on_stack,
                &mut sccs,
            );
        }
    }

    sccs
}

#[allow(clippy::too_many_arguments)]
fn tarjan_strongconnect<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    index_counter: &mut usize,
    stack: &mut Vec<&'a str>,
    indices: &mut HashMap<&'a str, usize>,
    lowlinks: &mut HashMap<&'a str, usize>,
    on_stack: &mut HashSet<&'a str>,
    sccs: &mut Vec<Vec<&'a str>>,
) {
    indices.insert(node, *index_counter);
    lowlinks.insert(node, *index_counter);
    *index_counter += 1;
    stack.push(node);
    on_stack.insert(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !indices.contains_key(neighbor) {
                tarjan_strongconnect(
                    neighbor,
                    adj,
                    index_counter,
                    stack,
                    indices,
                    lowlinks,
                    on_stack,
                    sccs,
                );
                let node_lowlink = *lowlinks.get(node).unwrap_or(&usize::MAX);
                let neighbor_lowlink = *lowlinks.get(neighbor).unwrap_or(&usize::MAX);
                lowlinks.insert(node, node_lowlink.min(neighbor_lowlink));
            } else if on_stack.contains(neighbor) {
                let node_lowlink = *lowlinks.get(node).unwrap_or(&usize::MAX);
                let neighbor_index = *indices.get(neighbor).unwrap_or(&usize::MAX);
                lowlinks.insert(node, node_lowlink.min(neighbor_index));
            }
        }
    }

    if lowlinks.get(node) == indices.get(node) {
        let mut scc = Vec::new();
        while let Some(top) = stack.pop() {
            on_stack.remove(top);
            scc.push(top);
            if top == node {
                break;
            }
        }
        sccs.push(scc);
    }
}

/// Find an actual cycle path within a strongly connected component
fn find_cycle_in_scc<'a>(
    scc: &[&'a str],
    adj: &HashMap<&'a str, Vec<&'a str>>,
) -> Option<Vec<&'a str>> {
    if scc.is_empty() {
        return None;
    }

    let scc_set: HashSet<&str> = scc.iter().copied().collect();

    let start = scc[0];
    let mut visited: HashSet<&str> = HashSet::new();
    let mut path: Vec<&'a str> = Vec::new();

    dfs_find_cycle(start, &scc_set, adj, &mut visited, &mut path)
}

fn dfs_find_cycle<'a>(
    node: &'a str,
    scc: &HashSet<&str>,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<&'a str>> {
    visited.insert(node);
    path.push(node);

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !scc.contains(neighbor) {
                continue;
            }

            if path.contains(neighbor) {
                if let Some(cycle_start) = path.iter().position(|&n| n == *neighbor) {
                    return Some(path[cycle_start..].to_vec());
                }
            } else if !visited.contains(neighbor) {
                if let Some(cycle) = dfs_find_cycle(neighbor, scc, adj, visited, path) {
                    return Some(cycle);
                }
            }
        }
    }

    path.pop();
    None
}

fn canonicalize_cycle(cycle: &[&str]) -> Vec<String> {
    if cycle.is_empty() {
        return Vec::new();
    }
    let n = cycle.len();
    let mut best_start = 0;
    for start in 1..n {
        for i in 0..n {
            let a = cycle[(best_start + i) % n];
            let b = cycle[(start + i) % n];
            match a.cmp(b) {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Greater => {
                    best_start = start;
                    break;
                }
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    (0..n)
        .map(|i| cycle[(best_start + i) % n].to_string())
        .collect()
}

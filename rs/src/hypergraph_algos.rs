use std::collections::{BTreeSet, HashSet, VecDeque};

use crate::grammar::Symbol;
use crate::hypergraph::{EdgeId, Hypergraph, NodeId};
use crate::semiring::Semiring;

pub fn topological_sort(hg: &mut Hypergraph) -> Vec<NodeId> {
    let mut sorted = Vec::new();
    let mut queue: VecDeque<NodeId> = VecDeque::new();

    // Start with nodes that have no incoming edges
    for (i, node) in hg.nodes.iter().enumerate() {
        if node.incoming.is_empty() {
            queue.push_back(NodeId(i));
        }
    }

    while let Some(nid) = queue.pop_front() {
        sorted.push(nid);
        let outgoing = hg.nodes[nid.0].outgoing.clone();
        for &eid in &outgoing {
            let edge = &mut hg.edges[eid.0];
            if edge.marked() {
                continue;
            }
            edge.mark += 1;
            if edge.marked() {
                let head = edge.head;
                // Check if all incoming edges of head are marked
                let all_marked = hg.nodes[head.0]
                    .incoming
                    .iter()
                    .all(|&ie| hg.edges[ie.0].marked());
                if all_marked {
                    queue.push_back(head);
                }
            }
        }
    }

    sorted
}

pub fn viterbi_path<S: Semiring>(hg: &mut Hypergraph) -> (Vec<EdgeId>, f64) {
    let toposorted = topological_sort(hg);

    // Init
    for node in &mut hg.nodes {
        node.score = S::null();
    }
    if let Some(&first) = toposorted.first() {
        hg.nodes[first.0].score = S::one();
    }

    let mut best_path: Vec<EdgeId> = Vec::new();

    for &nid in &toposorted {
        let incoming = hg.nodes[nid.0].incoming.clone();
        let mut best_edge: Option<EdgeId> = None;
        for &eid in &incoming {
            let edge = &hg.edges[eid.0];
            let mut s = S::one();
            for &tid in &edge.tails {
                s = S::multiply(s, hg.nodes[tid.0].score);
            }
            let candidate = S::multiply(s, edge.score);
            if hg.nodes[nid.0].score < candidate {
                best_edge = Some(eid);
            }
            hg.nodes[nid.0].score = S::add(hg.nodes[nid.0].score, candidate);
        }
        if let Some(e) = best_edge {
            best_path.push(e);
        }
    }

    let final_score = toposorted
        .last()
        .map(|&nid| hg.nodes[nid.0].score)
        .unwrap_or(0.0);

    (best_path, final_score)
}

pub fn derive(hg: &Hypergraph, path: &[EdgeId], cur: NodeId, carry: &mut Vec<String>) {
    // Find edge in path whose head matches cur
    let node = &hg.nodes[cur.0];
    let edge_id = path
        .iter()
        .find(|&&eid| {
            let e = &hg.edges[eid.0];
            let h = &hg.nodes[e.head.0];
            h.symbol == node.symbol && h.left == node.left && h.right == node.right
        })
        .expect("derive: no matching edge found");

    let edge = &hg.edges[edge_id.0];
    for sym in &edge.rule.target {
        match sym {
            Symbol::NT { index, .. } => {
                // Find which tail to recurse into using map
                let tail_idx = edge
                    .rule
                    .map
                    .iter()
                    .position(|&m| m == *index)
                    .expect("derive: NT index not found in map");
                derive(hg, path, edge.tails[tail_idx], carry);
            }
            Symbol::T(word) => {
                carry.push(word.clone());
            }
        }
    }
}

pub fn all_paths(hg: &mut Hypergraph) -> Vec<Vec<EdgeId>> {
    let toposorted = topological_sort(hg);

    let mut paths: Vec<Vec<EdgeId>> = vec![vec![]];

    for &nid in &toposorted {
        let incoming = hg.nodes[nid.0].incoming.clone();
        if incoming.is_empty() {
            continue;
        }
        let mut new_paths = Vec::new();
        while let Some(p) = paths.pop() {
            for &eid in &incoming {
                let mut np = p.clone();
                np.push(eid);
                new_paths.push(np);
            }
        }
        paths = new_paths;
    }

    // Dedup by reachable edge set
    let mut seen: HashSet<Vec<usize>> = HashSet::new();
    paths
        .into_iter()
        .filter(|p| {
            if p.is_empty() {
                return false;
            }
            let mut reachable = BTreeSet::new();
            mark_reachable(hg, p, *p.last().unwrap(), &mut reachable);
            let key: Vec<usize> = reachable.into_iter().map(|eid| eid.0).collect();
            seen.insert(key)
        })
        .collect()
}

fn mark_reachable(
    hg: &Hypergraph,
    path: &[EdgeId],
    edge_id: EdgeId,
    used: &mut BTreeSet<EdgeId>,
) {
    used.insert(edge_id);
    let edge = &hg.edges[edge_id.0];
    for &tail_nid in &edge.tails {
        // Find edge in path whose head is this tail node
        if let Some(&child_eid) = path.iter().find(|&&eid| hg.edges[eid.0].head == tail_nid) {
            if !used.contains(&child_eid) {
                mark_reachable(hg, path, child_eid, used);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph_io::read_hypergraph_from_json;
    use crate::semiring::ViterbiSemiring;

    #[test]
    fn test_viterbi_toy() {
        let mut hg = read_hypergraph_from_json("../example/toy/toy.json", true).unwrap();
        let (path, score) = viterbi_path::<ViterbiSemiring>(&mut hg);
        let mut carry = Vec::new();
        let last_head = hg.edges[path.last().unwrap().0].head;
        derive(&hg, &path, last_head, &mut carry);

        let translation = carry.join(" ");
        let log_score = score.ln();

        assert_eq!(translation, "i saw a small shell");
        assert!((log_score - (-0.5)).abs() < 1e-9);
    }

    #[test]
    fn test_all_paths_toy() {
        let mut hg = read_hypergraph_from_json("../example/toy/toy.json", true).unwrap();

        let paths = all_paths(&mut hg);
        // The toy hypergraph should have multiple distinct paths
        assert!(paths.len() > 1);

        // Collect all translations
        let mut translations: Vec<String> = Vec::new();
        for p in &paths {
            let mut carry = Vec::new();
            let last_head = hg.edges[p.last().unwrap().0].head;
            derive(&hg, p, last_head, &mut carry);
            translations.push(carry.join(" "));
        }

        assert!(translations.contains(&"i saw a small shell".to_string()));
        assert!(translations.contains(&"i saw a small house".to_string()));
        assert!(translations.contains(&"i saw a little shell".to_string()));
        assert!(translations.contains(&"i saw a little house".to_string()));
    }
}

use std::collections::HashMap;

use crate::grammar::{Grammar, Rule, Symbol};
use crate::hypergraph::{Hypergraph, NodeId};
use crate::parse::{Chart, visit};
use crate::sparse_vector::SparseVector;

pub fn chart_to_hg(chart: &Chart, n: usize, weights: &SparseVector, grammar: &Grammar) -> Hypergraph {
    let mut hg = Hypergraph::new();
    let mut seen: HashMap<String, NodeId> = HashMap::new();

    // Add root node with id=-1
    let root_nid = hg.add_node(-1, "root", -1, -1);

    let mut next_id: i64 = 0;

    // First pass: create nodes
    visit(1, 0, n, 0, |i, j| {
        for item in chart.at(i, j) {
            let lhs_sym = grammar.rules[item.rule_idx].lhs.nt_symbol();
            let key = format!("{},{},{}", lhs_sym, i, j);
            if !seen.contains_key(&key) {
                let nid = hg.add_node(next_id, lhs_sym, i as i32, j as i32);
                seen.insert(key, nid);
                next_id += 1;
            }
        }
    });

    // Second pass: create edges
    visit(1, 0, n, 0, |i, j| {
        for item in chart.at(i, j) {
            let rule = &grammar.rules[item.rule_idx];
            let head_key = format!("{},{},{}", rule.lhs.nt_symbol(), i, j);
            let head_nid = *seen.get(&head_key).unwrap();

            // Build tails
            let tails: Vec<NodeId> = if item.tail_spans.iter().all(|s| s.is_none())
                || item.tail_spans.iter().all(|s| {
                    s.as_ref()
                        .map_or(true, |sp| sp.left == -1 && sp.right == -1)
                })
            {
                if rule.rhs.iter().any(|s| s.is_nt()) {
                    build_tails(&rule.rhs, &item.tail_spans, &seen, root_nid)
                } else {
                    vec![root_nid]
                }
            } else {
                build_tails(&rule.rhs, &item.tail_spans, &seen, root_nid)
            };

            let score = weights.dot(&rule.f).exp();
            let rule_clone = Rule::new(
                rule.lhs.clone(),
                rule.rhs.clone(),
                rule.target.clone(),
                rule.map.clone(),
                rule.f.clone(),
            );

            hg.add_edge(head_nid, tails, score, rule.f.clone(), rule_clone);
        }
    });

    hg
}

fn build_tails(
    rhs: &[Symbol],
    tail_spans: &[Option<crate::parse::Span>],
    seen: &HashMap<String, NodeId>,
    root_nid: NodeId,
) -> Vec<NodeId> {
    let mut tails = Vec::new();
    let mut has_nt = false;
    for (idx, sym) in rhs.iter().enumerate() {
        if let Symbol::NT { symbol, .. } = sym {
            has_nt = true;
            if let Some(Some(span)) = tail_spans.get(idx) {
                if span.left >= 0 && span.right >= 0 {
                    let key = format!("{},{},{}", symbol, span.left, span.right);
                    if let Some(&nid) = seen.get(&key) {
                        tails.push(nid);
                    }
                }
            }
        }
    }
    if !has_nt {
        vec![root_nid]
    } else {
        tails
    }
}

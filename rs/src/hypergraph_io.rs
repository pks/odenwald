use std::fs;
use std::io;

use crate::grammar::Rule;
use crate::hypergraph::{Hypergraph, NodeId};
use crate::sparse_vector::SparseVector;

pub fn read_hypergraph_from_json(path: &str, log_weights: bool) -> io::Result<Hypergraph> {
    let content = fs::read_to_string(path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut hg = Hypergraph::new();

    let weights = match parsed.get("weights").and_then(|w| w.as_object()) {
        Some(obj) => SparseVector::from_hash(obj),
        None => SparseVector::new(),
    };

    // Add nodes
    if let Some(nodes) = parsed.get("nodes").and_then(|n| n.as_array()) {
        for n in nodes {
            let id = n["id"].as_i64().unwrap();
            let cat = n["cat"].as_str().unwrap_or("");
            let span = n["span"].as_array().unwrap();
            let left = span[0].as_i64().unwrap() as i32;
            let right = span[1].as_i64().unwrap() as i32;
            hg.add_node(id, cat, left, right);
        }
    }

    // Add edges
    if let Some(edges) = parsed.get("edges").and_then(|e| e.as_array()) {
        for e in edges {
            let head_id = e["head"].as_i64().unwrap();
            let head_nid = *hg.nodes_by_id.get(&head_id).unwrap();

            let tails: Vec<NodeId> = e["tails"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| *hg.nodes_by_id.get(&t.as_i64().unwrap()).unwrap())
                .collect();

            let f = match e.get("f").and_then(|f| f.as_object()) {
                Some(obj) => SparseVector::from_hash(obj),
                None => SparseVector::new(),
            };

            let rule_str = e["rule"].as_str().unwrap();
            let rule = Rule::from_str(rule_str);

            let score = if log_weights {
                weights.dot(&f).exp()
            } else {
                weights.dot(&f)
            };

            hg.add_edge(head_nid, tails, score, f, rule);
        }
    }

    Ok(hg)
}

pub fn write_hypergraph_to_json(hg: &Hypergraph, weights: &SparseVector) -> String {
    let mut json_s = String::from("{\n");
    json_s.push_str(&format!(
        "\"weights\":{},\n",
        serde_json::to_string(&weights.to_json()).unwrap()
    ));

    json_s.push_str("\"nodes\":\n[\n");
    let node_strs: Vec<String> = hg
        .nodes
        .iter()
        .map(|n| {
            format!(
                "{{ \"id\":{}, \"cat\":\"{}\", \"span\":[{},{}] }}",
                n.id,
                n.symbol.replace('"', "\\\""),
                n.left,
                n.right
            )
        })
        .collect();
    json_s.push_str(&node_strs.join(",\n"));
    json_s.push_str("\n],\n");

    json_s.push_str("\"edges\":\n[\n");
    let edge_strs: Vec<String> = hg
        .edges
        .iter()
        .map(|e| {
            let head_id = hg.nodes[e.head.0].id;
            let tail_ids: Vec<i64> = e.tails.iter().map(|t| hg.nodes[t.0].id).collect();
            format!(
                "{{ \"head\":{}, \"rule\":\"{}\", \"tails\":{:?}, \"f\":{} }}",
                head_id,
                e.rule.to_string().replace('"', "\\\""),
                tail_ids,
                serde_json::to_string(&e.f.to_json()).unwrap()
            )
        })
        .collect();
    json_s.push_str(&edge_strs.join(",\n"));
    json_s.push_str("\n]\n}\n");

    json_s
}

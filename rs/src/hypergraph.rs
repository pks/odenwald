use crate::grammar::Rule;
use crate::sparse_vector::SparseVector;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(pub usize);

#[derive(Debug, Clone)]
pub struct Node {
    pub id: i64,
    pub symbol: String,
    pub left: i32,
    pub right: i32,
    pub outgoing: Vec<EdgeId>,
    pub incoming: Vec<EdgeId>,
    pub score: f64,
}

impl Node {
    pub fn new(id: i64, symbol: &str, left: i32, right: i32) -> Self {
        Self {
            id,
            symbol: symbol.to_string(),
            left,
            right,
            outgoing: Vec::new(),
            incoming: Vec::new(),
            score: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub head: NodeId,
    pub tails: Vec<NodeId>,
    pub score: f64,
    pub f: SparseVector,
    pub mark: usize,
    pub rule: Rule,
}

impl Edge {
    pub fn arity(&self) -> usize {
        self.tails.len()
    }

    pub fn marked(&self) -> bool {
        self.arity() == self.mark
    }
}

#[derive(Debug)]
pub struct Hypergraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub nodes_by_id: std::collections::HashMap<i64, NodeId>,
}

impl Hypergraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            nodes_by_id: std::collections::HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: i64, symbol: &str, left: i32, right: i32) -> NodeId {
        let nid = NodeId(self.nodes.len());
        self.nodes.push(Node::new(id, symbol, left, right));
        self.nodes_by_id.insert(id, nid);
        nid
    }

    pub fn add_edge(
        &mut self,
        head: NodeId,
        tails: Vec<NodeId>,
        score: f64,
        f: SparseVector,
        rule: Rule,
    ) -> EdgeId {
        let eid = EdgeId(self.edges.len());
        self.edges.push(Edge {
            head,
            tails: tails.clone(),
            score,
            f,
            mark: 0,
            rule,
        });
        self.nodes[head.0].incoming.push(eid);
        for &t in &tails {
            self.nodes[t.0].outgoing.push(eid);
        }
        eid
    }

    pub fn node(&self, nid: NodeId) -> &Node {
        &self.nodes[nid.0]
    }

    pub fn edge(&self, eid: EdgeId) -> &Edge {
        &self.edges[eid.0]
    }

    pub fn reset(&mut self) {
        for e in &mut self.edges {
            e.mark = 0;
        }
    }
}

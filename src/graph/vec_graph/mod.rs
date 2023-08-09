// Building minimal Graph from the ground up.
// Minimize memory overhead.
// Support finding an edge in log(|E|) time.
// See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Can also implement a version with separate Node/Edge lists and Edge pointers forming AVL tree.

use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::{Eq, Ord};
use std::ops::{Index, IndexMut};
use std::slice::Iter;

use graph::indexing::{DefaultIx, IndexType, NodeIndex};

pub mod dot;
// use self::dot::Dot;

// Potential feature: Avoid binary search on filled entries, or "fill" almost filled entries?
#[derive(Deserialize, Serialize)]
pub struct Graph<N, E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, E: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    nodes: Vec<Node<N, E, Ix>>,
}

impl<N, E, Ix: IndexType> Graph<N, E, Ix>
where
    E: Eq + Ord + Copy,
{
    pub fn new() -> Self {
        let nodes: Vec<Node<N, E, Ix>> = Vec::new();
        Graph { nodes }
    }

    pub fn with_capacity(n_nodes: usize) -> Self {
        let nodes: Vec<Node<N, E, Ix>> = Vec::with_capacity(n_nodes);
        Graph { nodes }
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let node = Node::new(weight);
        let node_idx = NodeIndex::new(self.nodes.len());
        assert!(<Ix as IndexType>::max().index() == !0 || NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
        self.nodes.get(a.index()).map(|n| &n.weight)
    }

    pub fn set_node_weight(&mut self, a: NodeIndex<Ix>, value: N) {
        if let Some(ptr) = self.nodes.get_mut(a.index()) {
            ptr.weight = value
        }
    }

    pub fn clone_node(&mut self, a: NodeIndex<Ix>) -> NodeIndex<Ix>
    where
        N: Clone,
        E: Clone,
        Ix: Clone,
    {
        let node = self.nodes[a.index()].clone();
        let node_idx = NodeIndex::new(self.nodes.len());
        self.nodes.push(node);
        node_idx
    }

    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        self.nodes[a.index()].add_edge(weight, b)
    }

    pub fn edge_target(&self, a: NodeIndex<Ix>, weight: E) -> Option<NodeIndex<Ix>> {
        self.nodes[a.index()].edge_target(weight)
    }

    pub fn remove_edge(&mut self, a: NodeIndex<Ix>, weight: E) -> bool {
        self.nodes[a.index()].remove_edge(weight)
    }

    pub fn reroute_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        self.nodes[a.index()].reroute_edge(weight, b)
    }

    pub fn edges(&self, a: NodeIndex<Ix>) -> Iter<'_, Edge<E, Ix>> {
        self.nodes[a.index()].edges.iter()
    }

    pub fn n_edges(&self, a: NodeIndex<Ix>) -> usize {
        self.nodes[a.index()].edges.len()
    }

    pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix> {
        Neighbors::new(&self.nodes[a.index()])
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.nodes.iter().map(|x| x.edges.len()).sum()
    }
}

impl<N, E, Ix> Index<NodeIndex<Ix>> for Graph<N, E, Ix>
where
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

impl<N, E, Ix> IndexMut<NodeIndex<Ix>> for Graph<N, E, Ix>
where
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }
}

#[derive(Deserialize, Serialize)]
pub struct Node<N, E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, E: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: N,
    edges: Vec<Edge<E, Ix>>,
}

impl<N, E, Ix> Clone for Node<N, E, Ix>
where
    N: Clone,
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        Node {
            weight: self.weight.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl<N, E, Ix: IndexType> Node<N, E, Ix>
where
    E: Eq + Ord + Copy,
{
    pub fn new(weight: N) -> Self {
        let edges = Vec::new();
        Self { weight, edges }
    }

    pub fn add_edge(&mut self, weight: E, target: NodeIndex<Ix>) -> bool {
        if self.edges.is_empty() {
            let edge = Edge { weight, target };
            self.edges.push(edge);
            return true;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight == weight {
            return false;
        }
        let edge = Edge { weight, target };
        if weight < self.edges[idx].weight {
            self.edges.insert(idx, edge);
        } else {
            self.edges.insert(idx + 1, edge);
        }

        true
    }

    pub fn edge_target(&self, weight: E) -> Option<NodeIndex<Ix>> {
        if self.edges.is_empty() {
            return None;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight != weight {
            return None;
        }
        Some(self.edges[idx].target)
    }

    pub fn remove_edge(&mut self, weight: E) -> bool {
        if self.edges.is_empty() {
            return false;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight != weight {
            return false;
        }
        self.edges.remove(idx);
        true
    }

    pub fn reroute_edge(&mut self, weight: E, target: NodeIndex<Ix>) -> bool {
        if self.edges.is_empty() {
            return false;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight != weight {
            return false;
        }
        self.edges.get_mut(idx).expect("").set_target(target);
        true
    }

    fn _binary_search(&self, weight: E, l: usize, r: usize) -> usize {
        if l + 1 == r {
            return l;
        }
        let mid = (l + r) / 2;
        let mid_weight = self.edges[mid].weight;
        if weight < mid_weight {
            self._binary_search(weight, l, mid)
        } else {
            self._binary_search(weight, mid, r)
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Edge<E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "E: Serialize, Ix: Serialize",
        deserialize = "E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: E,
    target: NodeIndex<Ix>,
}

impl<E, Ix> Clone for Edge<E, Ix>
where
    E: Clone,
    Ix: Clone,
{
    fn clone(&self) -> Self {
        Edge {
            weight: self.weight.clone(),
            target: self.target.clone(),
        }
    }
}

impl<E, Ix: IndexType> Edge<E, Ix> {
    pub fn weight(&self) -> &E {
        &self.weight
    }

    pub fn target(&self) -> NodeIndex<Ix> {
        self.target
    }

    pub fn set_target(&mut self, target: NodeIndex<Ix>) {
        self.target = target;
    }
}

pub struct Neighbors<'a, E: 'a, Ix: 'a = DefaultIx> {
    edges: &'a Vec<Edge<E, Ix>>,
    next: usize,
}

impl<'a, E, Ix> Iterator for Neighbors<'a, E, Ix>
where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        if self.next < self.edges.len() {
            self.next += 1;
            return Some(self.edges[self.next - 1].target());
        }
        None
    }
}

impl<'a, E, Ix> Neighbors<'a, E, Ix>
where
    Ix: IndexType,
{
    pub fn new<N>(node: &'a Node<N, E, Ix>) -> Self {
        Self {
            edges: &node.edges,
            next: 0,
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use graph::indexing::{IndexType, NodeIndex};
    use graph::vec_graph::dot::Dot;
    use graph::vec_graph::Graph;

    use serde::{Deserialize, Serialize};

    #[test]
    fn test_create_graph() {
        let mut graph: Graph<u8, u16> = Graph::new();
        assert_eq!(graph.add_node(5).index(), 0);
        assert_eq!(graph.add_node(5).index(), 1);
    }

    fn weights<N, E, Ix>(graph: &Graph<N, E, Ix>, q: NodeIndex<Ix>) -> Vec<E>
    where
        E: Ord + Eq + Copy,
        Ix: IndexType,
    {
        graph.edges(q).map(|x| *x.weight()).collect::<Vec<_>>()
    }

    #[test]
    fn test_add_edge() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        let q3 = graph.add_node(3);

        assert!(graph.add_edge(q1, q2, 2));
        assert_eq!(weights(&graph, q1), vec![2]);
        assert!(!graph.add_edge(q1, q3, 2));
        assert!(graph.add_edge(q1, q3, 3));
        assert_eq!(weights(&graph, q1), vec![2, 3]);
        assert!(graph.add_edge(q1, q3, 4));
        assert_eq!(weights(&graph, q1), vec![2, 3, 4]);
        assert!(!graph.add_edge(q1, q3, 3));
        assert!(!graph.add_edge(q1, q2, 4));

        assert_eq!(graph.edge_target(q1, 2), Some(q2));
        assert_eq!(graph.edge_target(q1, 3), Some(q3));
        assert_eq!(graph.edge_target(q1, 7), None);

        assert_eq!(graph.n_edges(q1), 3);
    }

    #[test]
    fn test_add_edge_ba() {
        let mut graph: Graph<u8, char> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);

        assert!(graph.add_edge(q0, q1, 'b'));
        assert!(graph.add_edge(q0, q2, 'a'));

        // println!("{:?}", Dot::new(&graph));
        let q0_weights: Vec<_> = graph.edges(q0).map(|x| *x.weight()).collect();
        assert_eq!(q0_weights, vec!['a', 'b']);
    }

    #[test]
    fn test_remove_edge() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        assert!(!graph.remove_edge(q0, 2));
        assert!(graph.add_edge(q0, q1, 2));
        assert!(graph.remove_edge(q0, 2));
        assert!(!graph.remove_edge(q0, 2));
    }

    #[test]
    fn test_clone_node() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        graph.add_edge(q0, q1, 2);

        let q2 = graph.clone_node(q0);
        assert_eq!(*graph.node_weight(q2).unwrap(), 0_u8);
        assert_eq!(graph.edge_target(q2, 2), Some(q1));
    }

    #[test]
    fn test_reroute_edge() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        graph.add_edge(q0, q1, 2);
        assert!(graph.reroute_edge(q0, q2, 2));
        assert_eq!(graph.edge_target(q0, 2), Some(q2));
    }
}

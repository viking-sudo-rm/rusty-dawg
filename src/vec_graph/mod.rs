// Building minimal Graph from the ground up.
// Minimize memory overhead.
// Support finding an edge in log(|E|) time.
// See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Can also implement a version with separate Node/Edge lists and Edge pointers forming AVL tree.

use std::cmp::{Eq, Ord};
use std::ops::{Index, IndexMut};
use std::slice::Iter;

#[allow(dead_code)]
pub mod indexing;
use self::indexing::{DefaultIx, NodeIndex, IndexType};

pub mod dot;
// use self::dot::Dot;

pub struct Graph<N, E, Ix = DefaultIx> {
    nodes: Vec<Node<N, E, Ix>>,
}

impl<N, E: Eq + Ord + Copy, Ix: IndexType> Graph<N, E, Ix> {

    pub fn new() -> Self {
        let nodes: Vec<Node<N, E, Ix>> = Vec::new();
        Graph {nodes}
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

    // pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N> {
    //     self.nodes.get_mut(a.index()).map(|n| &mut n.weight)
    // }

    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        self.nodes[a.index()].add_edge(weight, b)
    }

    pub fn edge_target(&self, a: NodeIndex<Ix>, weight: E) -> Option<NodeIndex<Ix>> {
        self.nodes[a.index()].edge_target(weight)
    }

    pub fn remove_edge(&mut self, a: NodeIndex<Ix>, weight: E) -> bool {
        self.nodes[a.index()].remove_edge(weight)
    }

    pub fn edges(&self, a: NodeIndex<Ix>) -> Iter<'_, Edge<E, Ix>> {
        self.nodes[a.index()].edges.iter()
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
where Ix: IndexType {
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

impl<N, E, Ix> IndexMut<NodeIndex<Ix>> for Graph<N, E, Ix>
where Ix: IndexType {
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }
}

pub struct Node<N, E, Ix = DefaultIx> {
    pub weight: N,
    edges: Vec<Edge<E, Ix>>,
}

impl<N, E: Eq + Ord + Copy, Ix: IndexType> Node<N, E, Ix> {

    pub fn new(weight: N) -> Self {
        let edges = Vec::new();
        Self {weight, edges}
    }

    pub fn add_edge(&mut self, weight: E, target: NodeIndex<Ix>) -> bool {
        if self.edges.len() == 0 {
            let edge = Edge {weight, target};
            self.edges.push(edge);
            return true;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight == weight {
            return false;
        }
        let edge = Edge {weight, target};
        if weight < self.edges[idx].weight {
            self.edges.insert(idx, edge);
        } else {
            self.edges.insert(idx + 1, edge);
        }
        return true;
    }

    pub fn edge_target(&self, weight: E) -> Option<NodeIndex<Ix>> {
        if self.edges.len() == 0 {
            return None;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight != weight {
            return None;
        }
        return Some(self.edges[idx].target);
    }

    pub fn remove_edge(&mut self, weight: E) -> bool {
        if self.edges.len() == 0 {
            return false;
        }

        let idx = self._binary_search(weight, 0, self.edges.len());
        if self.edges[idx].weight != weight {
            return false;
        }
        self.edges.remove(idx);
        return true;
    }

    fn _binary_search(&self, weight: E, l: usize, r: usize) -> usize {
        if l + 1 == r {
            return l;
        }
        let mid = (l + r) / 2;
        let mid_weight = self.edges[mid].weight;
        if weight < mid_weight {
            return self._binary_search(weight, l, mid);
        } else {
            return self._binary_search(weight, mid, r);
        }
    }

}

pub struct Edge<E, Ix = DefaultIx> {
    pub weight: E,
    target: NodeIndex<Ix>,
}

impl<E, Ix: IndexType> Edge<E, Ix> {

    pub fn weight(&self) -> &E {
        &self.weight
    }

    pub fn target(&self) -> NodeIndex<Ix> {
        self.target
    }

}

pub struct Neighbors<'a, E: 'a, Ix: 'a = DefaultIx> {
    edges: &'a Vec<Edge<E, Ix>>,
    next: usize,
}

impl<'a, E, Ix> Iterator for Neighbors<'a, E, Ix>
where Ix: IndexType {
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
where Ix: IndexType {
    pub fn new<N>(node: &'a Node<N, E, Ix>) -> Self {
        Self {edges: &node.edges, next: 0}
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use vec_graph::Graph;
    use vec_graph::indexing::{NodeIndex, IndexType};
    use vec_graph::dot::Dot;

    #[test]
    fn test_create_graph() {
        let mut graph: Graph<u8, u16> = Graph::new();
        assert_eq!(graph.add_node(5).index(), 0);
        assert_eq!(graph.add_node(5).index(), 1);
    }

    fn weights<N, E, Ix>(graph: &Graph<N, E, Ix>, q: NodeIndex<Ix>) -> Vec<E>
    where E: Ord, E: std::cmp::Eq, E: std::marker::Copy, Ix: IndexType {
        graph.edges(q).map(|x| *x.weight()).collect::<Vec<_>>()
    }

    #[test]
    fn test_add_edge() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        let q3 = graph.add_node(3);

        assert_eq!(graph.add_edge(q1, q2, 2), true);
        assert_eq!(weights(&graph, q1), vec![2]);
        assert_eq!(graph.add_edge(q1, q3, 2), false);
        assert_eq!(graph.add_edge(q1, q3, 3), true);
        assert_eq!(weights(&graph, q1), vec![2, 3]);
        assert_eq!(graph.add_edge(q1, q3, 4), true);
        assert_eq!(weights(&graph, q1), vec![2, 3, 4]);
        assert_eq!(graph.add_edge(q1, q3, 3), false);
        assert_eq!(graph.add_edge(q1, q2, 4), false);

        assert_eq!(graph.edge_target(q1, 2), Some(q2));
        assert_eq!(graph.edge_target(q1, 3), Some(q3));
        assert_eq!(graph.edge_target(q1, 7), None);
    }

    #[test]
    fn test_add_edge_ba() {
        let mut graph: Graph<u8, char> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);

        assert_eq!(graph.add_edge(q0, q1, 'b'), true);
        assert_eq!(graph.add_edge(q0, q2, 'a'), true);

        // println!("{:?}", Dot::new(&graph));
        let q0_weights: Vec<_> = graph.edges(q0).map(|x| *x.weight()).collect();
        assert_eq!(q0_weights, vec!['a', 'b']);
    }

    #[test]
    fn test_remove_edge() {
        let mut graph: Graph<u8, u16> = Graph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        assert_eq!(graph.remove_edge(q0, 2), false);
        assert_eq!(graph.add_edge(q0, q1, 2), true);
        assert_eq!(graph.remove_edge(q0, 2), true);
        assert_eq!(graph.remove_edge(q0, 2), false);
    }

}
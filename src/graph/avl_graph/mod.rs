// Building minimal AvlGraph from the ground up.
// Minimize memory overhead.
// Support finding an edge in log(|E|) time.
// See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Can also implement a version with separate Node/Edge lists and Edge pointers forming AVL tree.

// https://stackoverflow.com/questions/7211806/how-to-implement-insertion-for-avl-tree-without-parent-pointer

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::clone::Clone;
use std::cmp::{Eq, Ord};

use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use disk_vec::DiskVec;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};

pub mod dot;

//#[derive(Default)]
pub struct AvlGraph<N, E, Ix = DefaultIx>
    where N: Serialize + DeserializeOwned + Default,
          Ix: Serialize + DeserializeOwned + Default,
          E: Serialize + DeserializeOwned + Default {
    // #[serde(bound(
    //     serialize = "N: Serialize, E: Serialize, Ix: Serialize",
    //     deserialize = "N: DeserializeOwned, E: DeserializeOwned, Ix: DeserializeOwned",
    // ))]
    nodes: DiskFVec<Node<N, Ix>>,
    edges: DiskVec<Edge<E, Ix>>,
}

impl<N, E, Ix: IndexType> AvlGraph<N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default + Eq + Ord + Copy + Debug,
        Ix: Serialize + DeserializeOwned + Default + Debug,
{
    pub fn new() -> Self {
        let nodes = DiskVec::new("data/nodes.bin").unwrap();
        let edges = DiskVec::new("data/edges.bin").unwrap();
        AvlGraph { nodes, edges }
    }

    pub fn with_capacity(n_nodes: usize, n_edges: usize) -> Self {
        let nodes = DiskVec::new("data/nodes.bin").unwrap();
        let edges = DiskVec::new("data/edges.bin").unwrap();
        AvlGraph { nodes, edges }
    }

    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let node = Node::new(weight);
        let node_idx = NodeIndex::new(self.nodes.len());
        assert!(<Ix as IndexType>::max_value().index() == !0 || NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N> {
        self.nodes.get(a.index()).map(|n| &n.weight).ok()
    }

    pub fn set_node_weight(&mut self, a: NodeIndex<Ix>, value: N) {
        // if let Some(ptr) = self.nodes.get_mut(a.index()) {
        //     ptr.weight = value;
        // }
        if let Some(node) = self.nodes.get(a.index()).ok() {
            let new_node = Node {
                weight: value,
                first_edge: node.first_edge,
            };
            self.nodes.set(a.index(), new_node);
        }
    }

    pub fn clone_node(&mut self, a: NodeIndex<Ix>) -> NodeIndex<Ix>
        where
            N: Clone,
            E: Clone,
            Ix: Clone,
    {
        let clone = Node::new(self.nodes[a.index()].weight.clone());
        let clone_idx = NodeIndex::new(self.nodes.len());
        self.nodes.push(clone);

        let first_source_idx = self.nodes[a.index()].first_edge;
        if first_source_idx == EdgeIndex::end() {
            return clone_idx;
        }

        let edge_to_clone = &self.edges[first_source_idx.index()];
        let first_clone_edge = Edge::new(edge_to_clone.weight, edge_to_clone.target);
        let first_clone_idx = EdgeIndex::new(self.edges.len());
        self.edges.push(first_clone_edge);
        self.nodes[clone_idx.index()].first_edge = first_clone_idx;
        self.clone_edges(first_source_idx, first_clone_idx);
        clone_idx
    }

    // The nodes that get passed in are the parents of the ones getting cloned.
    pub fn clone_edges(&mut self, old: EdgeIndex<Ix>, new: EdgeIndex<Ix>) {
        if old == EdgeIndex::end() {
            return;
        }
        let left = self.edges[old.index()].left;
        let right = self.edges[old.index()].right;

        if left != EdgeIndex::end() {
            let left_weight = self.edges[left.index()].weight;
            let left_target = self.edges[left.index()].target;
            let new_left_edge = Edge::new(left_weight, left_target);
            let new_left = EdgeIndex::new(self.edges.len());
            self.edges.push(new_left_edge);
            self.edges[new.index()].left = new_left;
            self.clone_edges(left, new_left);
        }

        if right != EdgeIndex::end() {
            let right_weight = self.edges[right.index()].weight;
            let right_target = self.edges[right.index()].target;
            let new_right_edge = Edge::new(right_weight, right_target);
            let new_right = EdgeIndex::new(self.edges.len());
            self.edges.push(new_right_edge);
            self.edges[new.index()].right = new_right;
            self.clone_edges(right, new_right);
        }
    }

    pub fn edge_weight(&self, edge: EdgeIndex<Ix>) -> Option<&E> {
        self.edges.get(edge.index()).map(|e| &e.weight).ok()
    }

    pub fn edge_tree_height(&self, node: NodeIndex<Ix>) -> usize {
        self.edge_tree_height_helper(self.nodes[node.index()].first_edge)
    }

    fn edge_tree_height_helper(&self, root: EdgeIndex<Ix>) -> usize {
        if root == EdgeIndex::end() {
            return 0;
        }
        std::cmp::max(
            self.edge_tree_height_helper(self.edges[root.index()].left),
            self.edge_tree_height_helper(self.edges[root.index()].right),
        ) + 1
    }

    pub fn balance_ratio(&self, node: NodeIndex<Ix>) -> f64 {
        (self.edge_tree_height(node) as f64) / (self.n_edges(node) as f64).log2().ceil()
    }

    // First result is either where weight was found or end; second is node above that (where to insert).
    fn binary_search(
        &self,
        edge: EdgeIndex<Ix>,
        last_edge: EdgeIndex<Ix>,
        weight: E,
    ) -> (EdgeIndex<Ix>, EdgeIndex<Ix>) {
        if edge == EdgeIndex::end() {
            return (edge, last_edge);
        }

        let edge_weight = self.edges[edge.index()].weight;
        if weight == edge_weight {
            (edge, last_edge)
        } else if weight < edge_weight {
            return self.binary_search(self.edges[edge.index()].left, edge, weight);
        } else {
            return self.binary_search(self.edges[edge.index()].right, edge, weight);
        }
    }

    // DONT USE THIS, here for legacy test reasons
    pub fn add_edge(
        &mut self,
        a: NodeIndex<Ix>,
        b: NodeIndex<Ix>,
        weight: E,
    ) -> Option<EdgeIndex<Ix>> {
        let edge = Edge::new(weight, b);
        let edge_idx = EdgeIndex::new(self.edges.len());

        // look for root, simple case where no root handled
        let first_edge = self.nodes[a.index()].first_edge;
        if first_edge == EdgeIndex::end() {
            self.nodes[a.index()].first_edge = edge_idx;
            self.edges.push(edge);
            return Some(edge_idx);
        }

        // binary search to find pointer where we insert new edge (edge and parent pointers)
        let (e, last_e) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e != EdgeIndex::end() {
            return None;
        }
        // weight of the parent
        let add_weight = self.edges[last_e.index()].weight;
        // weight less than parent, add left else right (the tree thing, no case where weights are equal)
        if weight < add_weight {
            self.edges[last_e.index()].left = edge_idx;
        } else {
            self.edges[last_e.index()].right = edge_idx;
        }
        // push this into the list of edges
        self.edges.push(edge);
        Some(edge_idx)
        // FIXME: Implement recursive version!!!
    }

    pub fn add_balanced_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) {
        // look for root, simple case where no root handled
        let first_edge = self.nodes[a.index()].first_edge;

        // recursive insert into AVL tree
        self.nodes[a.index()].first_edge = self.avl_insert_edge(first_edge, weight, b);
    }

    fn avl_insert_edge(
        &mut self,
        root_edge_idx: EdgeIndex<Ix>,
        weight: E,
        b: NodeIndex<Ix>,
    ) -> EdgeIndex<Ix> {
        // if we encounter null ptr, we add edge into AVL tree
        if root_edge_idx == EdgeIndex::end() {
            let edge = Edge::new(weight, b);
            self.edges.push(edge);
            return EdgeIndex::new(self.edges.len() - 1);
        }

        // keep recursing into the tree according to balance tree insert rule
        let root_edge_weight = self.edges[root_edge_idx.index()].weight;

        if weight < root_edge_weight {
            let init_left_idx: EdgeIndex<Ix> = self.edges[root_edge_idx.index()].left;
            let init_balance_factor: i8 = if init_left_idx == EdgeIndex::end() {
                0
            } else {
                self.edges[init_left_idx.index()].balance_factor
            };

            self.edges[root_edge_idx.index()].left = self.avl_insert_edge(init_left_idx, weight, b);

            let updated_left_idx = self.edges[root_edge_idx.index()].left;
            let updated_balance_factor = if updated_left_idx == EdgeIndex::end() {
                0
            } else {
                self.edges[updated_left_idx.index()].balance_factor
            };

            if init_balance_factor == 0
                && (init_left_idx == EdgeIndex::end()
                || updated_balance_factor == 1
                || updated_balance_factor == -1)
            {
                self.edges[root_edge_idx.index()].balance_factor += 1;
            }

            let current_balance_factor: i8 = self.edges[root_edge_idx.index()].balance_factor;
            if current_balance_factor == 2 {
                if updated_balance_factor == 1 {
                    return self.rotate_from_left(root_edge_idx);
                } else if updated_balance_factor == -1 {
                    return self.double_rotate_from_left(root_edge_idx);
                }
            }
        } else if root_edge_weight < weight {
            let init_right_idx: EdgeIndex<Ix> = self.edges[root_edge_idx.index()].right;
            let init_balance_factor: i8 = if init_right_idx == EdgeIndex::end() {
                0
            } else {
                self.edges[init_right_idx.index()].balance_factor
            };

            self.edges[root_edge_idx.index()].right =
                self.avl_insert_edge(init_right_idx, weight, b);

            let updated_right_idx = self.edges[root_edge_idx.index()].right;
            let updated_balance_factor = if updated_right_idx == EdgeIndex::end() {
                0
            } else {
                self.edges[updated_right_idx.index()].balance_factor
            };

            if init_balance_factor == 0
                && (init_right_idx == EdgeIndex::end()
                || updated_balance_factor == 1
                || updated_balance_factor == -1)
            {
                self.edges[root_edge_idx.index()].balance_factor -= 1;
            }

            let current_balance_factor: i8 = self.edges[root_edge_idx.index()].balance_factor;
            if current_balance_factor == -2 {
                if updated_balance_factor == -1 {
                    return self.rotate_from_right(root_edge_idx);
                } else if updated_balance_factor == 1 {
                    return self.double_rotate_from_right(root_edge_idx);
                }
            }
        }

        root_edge_idx
    }

    // AVL tree balance insert functions
    fn rotate_from_right(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        let p: EdgeIndex<Ix> = self.edges[node_ptr.index()].right;
        self.edges[node_ptr.index()].right = self.edges[p.index()].left;
        self.edges[p.index()].left = node_ptr;

        // update balance-factors
        // update rules taken from: https://cs.stackexchange.com/questions/48861/balance-factor-changes-after-local-rotations-in-avl-tree
        // p is l' and p.left (node_ptr) is n'
        // b(n') = b(n) + 1 - min(b(l), 0)
        // b(l') = b(l) + 1 + max(b(n'), 0)
        self.edges[node_ptr.index()].balance_factor +=
            1 - std::cmp::min(self.edges[p.index()].balance_factor, 0);
        self.edges[p.index()].balance_factor +=
            1 + std::cmp::max(self.edges[node_ptr.index()].balance_factor, 0);

        p
    }

    fn rotate_from_left(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        let p: EdgeIndex<Ix> = self.edges[node_ptr.index()].left;
        self.edges[node_ptr.index()].left = self.edges[p.index()].right;
        self.edges[p.index()].right = node_ptr;

        // update balance-factors
        // update rules taken from: https://cs.stackexchange.com/questions/48861/balance-factor-changes-after-local-rotations-in-avl-tree
        // p is l' and p.right (node_ptr) is n'
        // b(n') = b(n) - 1 - max(b(l), 0)
        // b(l') = b(l) - 1 + min(b(n'), 0)
        self.edges[node_ptr.index()].balance_factor -=
            1 + std::cmp::max(self.edges[p.index()].balance_factor, 0);
        self.edges[p.index()].balance_factor -=
            1 - std::cmp::min(self.edges[node_ptr.index()].balance_factor, 0);

        p
    }

    fn double_rotate_from_right(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        self.edges[node_ptr.index()].right =
            self.rotate_from_left(self.edges[node_ptr.index()].right);
        self.rotate_from_right(node_ptr)
    }

    fn double_rotate_from_left(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        self.edges[node_ptr.index()].left =
            self.rotate_from_right(self.edges[node_ptr.index()].left);
        self.rotate_from_left(node_ptr)
    }

    pub fn edge_target(&self, a: NodeIndex<Ix>, weight: E) -> Option<NodeIndex<Ix>> {
        let first_edge = self.nodes[a.index()].first_edge;
        if first_edge == EdgeIndex::end() {
            return None;
        }

        let (e, _last_e) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e == EdgeIndex::end() {
            return None;
        }
        Some(self.edges[e.index()].target())
    }

    pub fn reroute_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        let first_edge = self.nodes[a.index()].first_edge;
        if first_edge == EdgeIndex::end() {
            return false;
        }

        let (e, _) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e == EdgeIndex::end() {
            return false;
        }
        self.edges[e.index()].set_target(b);
        true
    }

    pub fn n_edges(&self, a: NodeIndex<Ix>) -> usize {
        let mut stack = vec![self.nodes[a.index()].first_edge];
        let mut count = 0;
        while let Some(top) = stack.pop() {
            if top == EdgeIndex::end() {
                continue;
            }
            count += 1;
            stack.push(self.edges[top.index()].left);
            stack.push(self.edges[top.index()].right);
        }
        count
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node: NodeIndex<Ix>) -> Neighbors<N, E, Ix> {
        Neighbors::new(self, node)
    }

    pub fn edges(&self, edges: NodeIndex<Ix>) -> Edges<N, E, Ix> {
        Edges::new(self, edges)
    }
}

impl<N, E, Ix> Index<NodeIndex<Ix>> for AvlGraph<N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

impl<N, E, Ix> IndexMut<NodeIndex<Ix>> for AvlGraph<N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }
}

pub struct Neighbors<'a, N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    edges: Edges<'a, N, E, Ix>,
}

impl<'a, N, E, Ix> Iterator for Neighbors<'a, N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.edges.next().map(|edge| edge.target())
    }
}

impl<'a, N, E, Ix> Neighbors<'a, N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    pub fn new(graph: &'a AvlGraph<N, E, Ix>, node: NodeIndex<Ix>) -> Self {
        let edges = Edges::new(graph, node);
        Self { edges }
    }
}

pub struct Edges<'a, N, E, Ix>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    graph: &'a AvlGraph<N, E, Ix>,
    stack: Vec<EdgeIndex<Ix>>,
}

impl<'a, N, E, Ix> Iterator for Edges<'a, N, E, Ix>
    where
    N: Serialize + DeserializeOwned + Default,
    E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    type Item = &'a Edge<E, Ix>;

    // Note that this was hurting performance a lot before!
    // Be careful using a vector as a stack.
    // https://stackoverflow.com/questions/40848918/are-there-queue-and-stack-collections-in-rust
    fn next(&mut self) -> Option<&'a Edge<E, Ix>> {
        // Is this pop_back()????
        match self.stack.pop() {
            None => None,
            Some(idx) => {
                if idx == EdgeIndex::end() {
                    // Only hit for an empty tree.
                    return None;
                }

                let left = self.graph.edges[idx.index()].left;
                if left != EdgeIndex::end() {
                    self.stack.push(left);
                }
                let right = self.graph.edges[idx.index()].right;
                if right != EdgeIndex::end() {
                    self.stack.push(right);
                }
                Some(&self.graph.edges[idx.index()])
            }
        }
    }
}

impl<'a, N, E, Ix> Edges<'a, N, E, Ix>
    where
    N: Serialize + DeserializeOwned + Default,
    E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default + IndexType,
{
    pub fn new(graph: &'a AvlGraph<N, E, Ix>, node: NodeIndex<Ix>) -> Self {
        let root = graph.nodes[node.index()].first_edge;
        let stack = vec![root];
        // let mut stack = LinkedList::new();
        // stack.push_back(root);
        Self { graph, stack }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Node<N, Ix = DefaultIx> {
    #[serde(bound(
    serialize = "N: Serialize, Ix: Serialize",
    deserialize = "N: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: N,
    pub first_edge: EdgeIndex<Ix>,
}

impl<N, Ix> Clone for Node<N, Ix>
    where
        N: Clone,
        Ix: Clone,
{
    fn clone(&self) -> Self {
        Node {
            weight: self.weight.clone(),
            first_edge: self.first_edge.clone(),
        }
    }
}

impl<N, Ix: IndexType> Node<N, Ix> {
    pub fn new(weight: N) -> Self {
        Self {
            weight,
            first_edge: EdgeIndex::end(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Edge<E, Ix = DefaultIx> {
    #[serde(bound(
    serialize = "E: Serialize, Ix: Serialize",
    deserialize = "E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    pub weight: E,
    target: NodeIndex<Ix>,
    pub left: EdgeIndex<Ix>,
    pub right: EdgeIndex<Ix>,
    pub balance_factor: i8,
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
            left: self.left.clone(),
            right: self.right.clone(),
            balance_factor: self.balance_factor,
        }
    }
}

impl<E, Ix: IndexType> Edge<E, Ix> {
    pub fn new(weight: E, target: NodeIndex<Ix>) -> Self {
        Edge {
            weight,
            target,
            left: EdgeIndex::end(),
            right: EdgeIndex::end(),
            balance_factor: 0,
        }
    }

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

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use graph::avl_graph::AvlGraph;
    use graph::indexing::{EdgeIndex, IndexType, NodeIndex};
    // use graph::avl_graph::dot::Dot;

    use serde::{Deserialize, Serialize};

    #[test]
    fn test_create_graph() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        assert_eq!(graph.add_node(5).index(), 0);
        assert_eq!(graph.add_node(5).index(), 1);
    }

    // fn weights<N, E, Ix>(graph: &AvlGraph<N, E, Ix>, q: NodeIndex<Ix>) -> Vec<E>
    // where E: Ord + Eq + Copy, Ix: IndexType {
    //     graph.edges(q).map(|x| *x.weight()).collect::<Vec<_>>()
    // }

    #[test]
    fn test_add_edge() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        let q3 = graph.add_node(3);

        assert_eq!(graph.add_edge(q1, q2, 2), Some(EdgeIndex::new(0)));
        // assert_eq!(weights(&graph, q1), vec![2]);
        assert_eq!(graph.add_edge(q1, q3, 2), None);
        assert_eq!(graph.add_edge(q1, q3, 3), Some(EdgeIndex::new(1)));
        // assert_eq!(weights(&graph, q1), vec![2, 3]);
        assert_eq!(graph.add_edge(q1, q3, 4), Some(EdgeIndex::new(2)));
        // assert_eq!(weights(&graph, q1), vec![2, 3, 4]);
        assert_eq!(graph.add_edge(q1, q3, 3), None);
        assert_eq!(graph.add_edge(q1, q2, 4), None);

        assert_eq!(graph.edge_target(q1, 2), Some(q2));
        assert_eq!(graph.edge_target(q1, 3), Some(q3));
        assert_eq!(graph.edge_target(q1, 7), None);

        assert_eq!(graph.n_edges(q1), 3);
    }

    #[test]
    fn test_add_edge_ba() {
        let mut graph: AvlGraph<u8, char> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);

        assert_eq!(graph.add_edge(q0, q1, 'b'), Some(EdgeIndex::new(0)));
        assert_eq!(graph.add_edge(q0, q2, 'a'), Some(EdgeIndex::new(1)));

        // println!("{:?}", Dot::new(&graph));
        // let q0_weights: Vec<_> = graph.edges(q0).map(|x| *x.weight()).collect();
        // assert_eq!(q0_weights, vec!['a', 'b']);
    }

    // #[test]
    // fn test_remove_edge() {
    //     let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
    //     let q0 = graph.add_node(0);
    //     let q1 = graph.add_node(1);

    //     assert_eq!(graph.remove_edge(q0, 2), false);
    //     assert_eq!(graph.add_edge(q0, q1, 2), true);
    //     assert_eq!(graph.remove_edge(q0, 2), true);
    //     assert_eq!(graph.remove_edge(q0, 2), false);
    // }

    #[test]
    fn test_rotate_from_right() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        let mut root = graph.add_edge(q0, q1, 1).unwrap();
        let e1 = graph.add_edge(q0, q1, 0).unwrap();
        let e2 = graph.add_edge(q0, q1, 3).unwrap();
        let e3 = graph.add_edge(q0, q1, 2).unwrap();
        let e4 = graph.add_edge(q0, q1, 4).unwrap();

        graph.edges[root.index()].balance_factor = -1;
        graph.edges[e1.index()].balance_factor = 0;
        graph.edges[e2.index()].balance_factor = 0;
        graph.edges[e3.index()].balance_factor = 0;
        graph.edges[e4.index()].balance_factor = 0;

        root = graph.rotate_from_right(root);

        let left = graph.edges[root.index()].left;
        let right = graph.edges[root.index()].right;

        assert_eq!(graph.edges[root.index()].weight, 3);
        assert_eq!(graph.edges[left.index()].weight, 1);
        assert_eq!(graph.edges[right.index()].weight, 4);

        assert_eq!(graph.edges[root.index()].balance_factor, 1);
        assert_eq!(graph.edges[left.index()].balance_factor, 0);
        assert_eq!(graph.edges[right.index()].balance_factor, 0);
    }

    #[test]
    fn test_rotate_from_left() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        let mut root = graph.add_edge(q0, q1, 3).unwrap();
        let e1 = graph.add_edge(q0, q1, 1).unwrap();
        let e2 = graph.add_edge(q0, q1, 4).unwrap();
        let e3 = graph.add_edge(q0, q1, 0).unwrap();
        let e4 = graph.add_edge(q0, q1, 2).unwrap();

        graph.edges[root.index()].balance_factor = 1;
        graph.edges[e1.index()].balance_factor = 0;
        graph.edges[e2.index()].balance_factor = 0;
        graph.edges[e3.index()].balance_factor = 0;
        graph.edges[e4.index()].balance_factor = 0;

        root = graph.rotate_from_left(root);

        let left = graph.edges[root.index()].left;
        let right = graph.edges[root.index()].right;

        assert_eq!(graph.edges[root.index()].weight, 1);
        assert_eq!(graph.edges[left.index()].weight, 0);
        assert_eq!(graph.edges[right.index()].weight, 3);

        assert_eq!(graph.edges[root.index()].balance_factor, -1);
        assert_eq!(graph.edges[left.index()].balance_factor, 0);
        assert_eq!(graph.edges[right.index()].balance_factor, 0);
    }

    #[test]
    fn test_add_balanced_edge() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        let q3 = graph.add_node(3);

        graph.add_balanced_edge(q1, q2, 2);
        graph.add_balanced_edge(q1, q3, 2);
        graph.add_balanced_edge(q1, q3, 3);

        graph.add_balanced_edge(q1, q3, 4);
        graph.add_balanced_edge(q1, q3, 3);
        graph.add_balanced_edge(q1, q2, 4);

        for idx in 5..16 {
            let q = graph.add_node(idx);
            graph.add_balanced_edge(q1, q, idx.into());
        }

        assert_eq!(graph.edge_target(q1, 2), Some(q2));
        assert_eq!(graph.edge_target(q1, 3), Some(q3));
        assert_eq!(graph.edge_target(q1, 42), None);

        assert_eq!(graph.n_edges(q1), 14);
        assert_eq!(graph.edge_tree_height(q1), 4)
    }

    #[test]
    fn test_add_balanced_edge_left_branching() {
        let mut graph: AvlGraph<u8, u64> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        for idx in (0..127).rev() {
            graph.add_balanced_edge(q0, q1, idx);
        }
        assert_eq!(graph.n_edges(q0), 127);
        assert_eq!(graph.edge_tree_height(q0), 7)
    }

    #[test]
    fn test_tree_construction() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);

        graph.add_balanced_edge(q1, q0, 0);
        graph.add_balanced_edge(q1, q0, 1);

        let mut root = graph.nodes[q1.index()].first_edge;
        let mut left: EdgeIndex = graph.edges[root.index()].left;
        let mut right: EdgeIndex = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, -1);
        assert_eq!(left, EdgeIndex::end());
        assert_eq!(graph.edges[right.index()].weight, 1);

        graph.add_balanced_edge(q1, q0, 2);

        root = graph.nodes[q1.index()].first_edge;
        left = graph.edges[root.index()].left;
        right = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, 0);
        assert_eq!(graph.edges[root.index()].weight, 1);
        assert_eq!(graph.edges[left.index()].weight, 0);
        assert_eq!(graph.edges[right.index()].weight, 2);
    }

    #[test]
    fn test_clone_node() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        graph.add_edge(q0, q1, 2);

        let q2 = graph.clone_node(q0);
        assert_eq!(*graph.node_weight(q2).unwrap(), 0_u8);
        assert_eq!(graph.edge_target(q2, 2), Some(q1));
    }

    #[test]
    fn test_reroute_edge() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        let q2 = graph.add_node(2);
        graph.add_edge(q0, q1, 2);
        assert!(graph.reroute_edge(q0, q2, 2));
        assert_eq!(graph.edge_target(q0, 2), Some(q2));
    }

    #[test]
    fn test_edges_iterator() {
        let mut graph: AvlGraph<u8, u32> = AvlGraph::new();
        let q0 = graph.add_node(0);
        let q1 = graph.add_node(1);
        for idx in 0..7 {
            graph.add_balanced_edge(q0, q1, idx);
        }
        let edges: Vec<_> = graph
            .edges(q0)
            .map(|x| (*x.weight(), x.target().index()))
            .collect();
        assert_eq!(
            edges,
            vec![(3, 1), (5, 1), (6, 1), (4, 1), (1, 1), (2, 1), (0, 1)]
        );

        assert_eq!(graph.balance_ratio(q0), 1.0);
        // FIXME: But stilll take the time tho
    }
}

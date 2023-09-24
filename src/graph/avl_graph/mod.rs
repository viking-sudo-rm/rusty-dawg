// Building minimal AvlGraph from the ground up.
// Minimize memory overhead.
// Support finding an edge in log(|E|) time.
// See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Can also implement a version with separate Node/Edge lists and Edge pointers forming AVL tree.

// https://stackoverflow.com/questions/7211806/how-to-implement-insertion-for-avl-tree-without-parent-pointer

use std::clone::Clone;
use std::cmp::{Eq, Ord};

use std::marker::PhantomData;

use std::cmp::{max, min};
use std::fmt::Debug;

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use weight::Weight;

pub mod edge;
pub mod node;
mod serde;

use graph::avl_graph::edge::{Edge, EdgeMutRef, EdgeRef};
use graph::avl_graph::node::{Node, NodeMutRef, NodeRef};

use graph::memory_backing::ram_backing::RamBacking;
use graph::memory_backing::vec_backing::VecBacking;
use graph::memory_backing::MemoryBacking;

#[derive(Default)]
pub struct AvlGraph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    nodes: Mb::VecN,
    edges: Mb::VecE,
    mb: Mb,
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> AvlGraph<N, E, Ix>
where
    E: Eq + Ord + Copy + Debug,
    Ix: IndexType,
    N: Weight,
{
    pub fn new() -> Self {
        let mb: RamBacking<N, E, Ix> = RamBacking::default();
        Self::new_mb(mb)
    }
}

impl<N, E, Ix, Mb> AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Eq + Ord + Copy + Debug,
    Ix: IndexType,
{
    pub fn new_mb(mb: Mb) -> Self {
        let nodes = mb.new_node_vec(None);
        let edges = mb.new_edge_vec(None);
        AvlGraph {
            nodes,
            edges,
            mb,
            marker: PhantomData,
        }
    }

    pub fn with_capacity_mb(mb: Mb, n_nodes: usize, n_edges: usize) -> Self {
        let nodes = mb.new_node_vec(Some(n_nodes));
        let edges = mb.new_edge_vec(Some(n_edges));
        AvlGraph {
            nodes,
            edges,
            mb,
            marker: PhantomData,
        }
    }
}

impl<N, E, Ix, Mb> AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Eq + Ord + Copy + Debug,
    N: Weight,
    Ix: IndexType,
{
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let node = Node::new(weight);
        let node_idx = NodeIndex::new(self.nodes.len());
        assert!(<Ix as IndexType>::max_value().index() == !0 || NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    pub fn clone_node(&mut self, a: NodeIndex<Ix>) -> NodeIndex<Ix>
    where
        N: Clone,
        E: Clone,
        Ix: Clone,
        Mb::EdgeRef: Copy,
    {
        let clone = Node::new(self.nodes.index(a.index()).get_weight().clone());
        let clone_idx = NodeIndex::new(self.nodes.len());
        self.nodes.push(clone);

        let first_source_idx = self.nodes.index(a.index()).get_first_edge();
        if first_source_idx == EdgeIndex::end() {
            return clone_idx;
        }

        let edge_to_clone = &self.edges.index(first_source_idx.index());
        let first_clone_edge = Edge::new(edge_to_clone.get_weight(), edge_to_clone.get_target());
        let first_clone_idx = EdgeIndex::new(self.edges.len());
        self.edges.push(first_clone_edge);
        self.nodes
            .index_mut(clone_idx.index())
            .set_first_edge(first_clone_idx);
        self.clone_edges(first_source_idx, first_clone_idx);
        clone_idx
    }

    // The nodes that get passed in are the parents of the ones getting cloned.
    pub fn clone_edges(&mut self, old: EdgeIndex<Ix>, new: EdgeIndex<Ix>) {
        if old == EdgeIndex::end() {
            return;
        }
        let left = self.edges.index(old.index()).get_left();
        let right = self.edges.index(old.index()).get_right();

        if left != EdgeIndex::end() {
            let left_weight = self.edges.index(left.index()).get_weight();
            let left_target = self.edges.index(left.index()).get_target();
            let new_left_edge = Edge::new(left_weight, left_target);
            let new_left = EdgeIndex::new(self.edges.len());
            self.edges.push(new_left_edge);
            self.edges.index_mut(new.index()).set_left(new_left);
            self.clone_edges(left, new_left);
        }

        if right != EdgeIndex::end() {
            let right_weight = self.edges.index(right.index()).get_weight();
            let right_target = self.edges.index(right.index()).get_target();
            let new_right_edge = Edge::new(right_weight, right_target);
            let new_right = EdgeIndex::new(self.edges.len());
            self.edges.push(new_right_edge);
            self.edges.index_mut(new.index()).set_right(new_right);
            self.clone_edges(right, new_right);
        }
    }

    pub fn edge_tree_height(&self, node: NodeIndex<Ix>) -> usize {
        self.edge_tree_height_helper(self.nodes.index(node.index()).get_first_edge())
    }

    fn edge_tree_height_helper(&self, root: EdgeIndex<Ix>) -> usize {
        if root == EdgeIndex::end() {
            return 0;
        }
        std::cmp::max(
            self.edge_tree_height_helper(self.edges.index(root.index()).get_left()),
            self.edge_tree_height_helper(self.edges.index(root.index()).get_right()),
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

        let edge_weight = self.edges.index(edge.index()).get_weight();
        if weight == edge_weight {
            (edge, last_edge)
        } else if weight < edge_weight {
            return self.binary_search(self.edges.index(edge.index()).get_left(), edge, weight);
        } else {
            return self.binary_search(self.edges.index(edge.index()).get_right(), edge, weight);
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
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            self.nodes.index_mut(a.index()).set_first_edge(edge_idx);
            self.edges.push(edge);
            return Some(edge_idx);
        }

        // binary search to find pointer where we insert new edge (edge and parent pointers)
        let (e, last_e) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e != EdgeIndex::end() {
            return None;
        }
        // weight of the parent
        let add_weight = self.edges.index(last_e.index()).get_weight();
        // weight less than parent, add left else right (the tree thing, no case where weights are equal)
        if weight < add_weight {
            self.edges.index_mut(last_e.index()).set_left(edge_idx);
        } else {
            self.edges.index_mut(last_e.index()).set_right(edge_idx);
        }
        // push this into the list of edges
        self.edges.push(edge);
        Some(edge_idx)
        // FIXME: Implement recursive version!!!
    }

    pub fn add_balanced_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) {
        // look for root, simple case where no root handled
        let first_edge = self.nodes.index(a.index()).get_first_edge();

        // recursive insert into AVL tree
        let new_first_edge = self.avl_insert_edge(first_edge, weight, b);
        self.nodes
            .index_mut(a.index())
            .set_first_edge(new_first_edge);
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
        let root_edge_weight = self.edges.index(root_edge_idx.index()).get_weight();

        if weight < root_edge_weight {
            let init_left_idx: EdgeIndex<Ix> = self.edges.index(root_edge_idx.index()).get_left();
            let init_balance_factor: i8 = if init_left_idx == EdgeIndex::end() {
                0
            } else {
                self.edges.index(init_left_idx.index()).get_balance_factor()
            };

            let new_left = self.avl_insert_edge(init_left_idx, weight, b);
            self.edges
                .index_mut(root_edge_idx.index())
                .set_left(new_left);

            let updated_left_idx = self.edges.index(root_edge_idx.index()).get_left();
            let updated_balance_factor = if updated_left_idx == EdgeIndex::end() {
                0
            } else {
                self.edges
                    .index(updated_left_idx.index())
                    .get_balance_factor()
            };

            if init_balance_factor == 0
                && (init_left_idx == EdgeIndex::end()
                    || updated_balance_factor == 1
                    || updated_balance_factor == -1)
            {
                let bf = self.edges.index(root_edge_idx.index()).get_balance_factor();
                self.edges
                    .index_mut(root_edge_idx.index())
                    .set_balance_factor(bf + 1);
            }

            let current_balance_factor: i8 =
                self.edges.index(root_edge_idx.index()).get_balance_factor();
            if current_balance_factor == 2 {
                if updated_balance_factor == 1 {
                    return self.rotate_from_left(root_edge_idx);
                } else if updated_balance_factor == -1 {
                    return self.double_rotate_from_left(root_edge_idx);
                }
            }
        } else if root_edge_weight < weight {
            let init_right_idx: EdgeIndex<Ix> = self.edges.index(root_edge_idx.index()).get_right();
            let init_balance_factor: i8 = if init_right_idx == EdgeIndex::end() {
                0
            } else {
                self.edges
                    .index(init_right_idx.index())
                    .get_balance_factor()
            };

            let new_right = self.avl_insert_edge(init_right_idx, weight, b);
            self.edges
                .index_mut(root_edge_idx.index())
                .set_right(new_right);

            let updated_right_idx = self.edges.index(root_edge_idx.index()).get_right();
            let updated_balance_factor = if updated_right_idx == EdgeIndex::end() {
                0
            } else {
                self.edges
                    .index(updated_right_idx.index())
                    .get_balance_factor()
            };

            if init_balance_factor == 0
                && (init_right_idx == EdgeIndex::end()
                    || updated_balance_factor == 1
                    || updated_balance_factor == -1)
            {
                let bf = self.edges.index(root_edge_idx.index()).get_balance_factor();
                self.edges
                    .index_mut(root_edge_idx.index())
                    .set_balance_factor(bf - 1);
            }

            let current_balance_factor: i8 =
                self.edges.index(root_edge_idx.index()).get_balance_factor();
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
        let p: EdgeIndex<Ix> = self.edges.index(node_ptr.index()).get_right();
        let p_left = self.edges.index(p.index()).get_left();
        self.edges.index_mut(node_ptr.index()).set_right(p_left);
        self.edges.index_mut(p.index()).set_left(node_ptr);

        // update balance-factors
        // update rules taken from: https://cs.stackexchange.com/questions/48861/balance-factor-changes-after-local-rotations-in-avl-tree
        // p is l' and p.left (node_ptr) is n'
        // b(n') = b(n) + 1 - min(b(l), 0)
        // b(l') = b(l) + 1 + max(b(n'), 0)
        let node_bf = self.edges.index(node_ptr.index()).get_balance_factor();
        let p_bf = self.edges.index(p.index()).get_balance_factor();
        self.edges
            .index_mut(node_ptr.index())
            .set_balance_factor(node_bf + 1 - min(p_bf, 0));
        let node_bf2 = self.edges.index(node_ptr.index()).get_balance_factor();
        let p_bf2 = self.edges.index(p.index()).get_balance_factor();
        self.edges
            .index_mut(p.index())
            .set_balance_factor(p_bf2 + 1 + max(node_bf2, 0));

        p
    }

    fn rotate_from_left(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        let p: EdgeIndex<Ix> = self.edges.index(node_ptr.index()).get_left();
        let p_right = self.edges.index(p.index()).get_right();
        self.edges.index_mut(node_ptr.index()).set_left(p_right);
        self.edges.index_mut(p.index()).set_right(node_ptr);

        // update balance-factors
        // update rules taken from: https://cs.stackexchange.com/questions/48861/balance-factor-changes-after-local-rotations-in-avl-tree
        // p is l' and p.right (node_ptr) is n'
        // b(n') = b(n) - 1 - max(b(l), 0)
        // b(l') = b(l) - 1 + min(b(n'), 0)
        let node_bf = self.edges.index(node_ptr.index()).get_balance_factor();
        let p_bf = self.edges.index(p.index()).get_balance_factor();
        self.edges
            .index_mut(node_ptr.index())
            .set_balance_factor(node_bf - 1 - max(p_bf, 0));
        let node_bf2 = self.edges.index(node_ptr.index()).get_balance_factor();
        let p_bf2 = self.edges.index(p.index()).get_balance_factor();
        self.edges
            .index_mut(p.index())
            .set_balance_factor(p_bf2 - 1 + min(node_bf2, 0));

        p
    }

    fn double_rotate_from_right(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        let new_right = self.rotate_from_left(self.edges.index(node_ptr.index()).get_right());
        self.edges.index_mut(node_ptr.index()).set_right(new_right);
        self.rotate_from_right(node_ptr)
    }

    fn double_rotate_from_left(&mut self, node_ptr: EdgeIndex<Ix>) -> EdgeIndex<Ix> {
        let new_left = self.rotate_from_right(self.edges.index(node_ptr.index()).get_left());
        self.edges.index_mut(node_ptr.index()).set_left(new_left);
        self.rotate_from_left(node_ptr)
    }

    pub fn edge_target(&self, a: NodeIndex<Ix>, weight: E) -> Option<NodeIndex<Ix>> {
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            return None;
        }

        let (e, _last_e) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e == EdgeIndex::end() {
            return None;
        }
        Some(self.edges.index(e.index()).get_target())
    }

    pub fn reroute_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            return false;
        }

        let (e, _) = self.binary_search(first_edge, EdgeIndex::end(), weight);
        if e == EdgeIndex::end() {
            return false;
        }
        self.edges.index_mut(e.index()).set_target(b);
        true
    }

    pub fn n_edges(&self, a: NodeIndex<Ix>) -> usize {
        let mut stack = vec![self.nodes.index(a.index()).get_first_edge()];
        let mut count = 0;
        while let Some(top) = stack.pop() {
            if top == EdgeIndex::end() {
                continue;
            }
            count += 1;
            stack.push(self.edges.index(top.index()).get_left());
            stack.push(self.edges.index(top.index()).get_right());
        }
        count
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn neighbors(&self, node: NodeIndex<Ix>) -> Neighbors<N, E, Ix, Mb> {
        Neighbors::new(self, node)
    }

    pub fn edges(&self, edges: NodeIndex<Ix>) -> Edges<N, E, Ix, Mb> {
        Edges::new(self, edges)
    }

    // We can't use standard indexing because we have custom reference types.

    pub fn get_node(&self, node: NodeIndex<Ix>) -> Mb::NodeRef {
        self.nodes.index(node.index())
    }

    // We can't use mutable indexing because we return custom MutNode, not &mut Node.
    pub fn get_node_mut(&mut self, node: NodeIndex<Ix>) -> Mb::NodeMutRef {
        self.nodes.index_mut(node.index())
    }
}

pub struct Neighbors<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    edges: Edges<'a, N, E, Ix, Mb>,
}

impl<'a, N, E, Ix, Mb> Iterator for Neighbors<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.edges.next().map(|edge| edge.get_target())
    }
}

impl<'a, N, E, Ix, Mb> Neighbors<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    pub fn new(graph: &'a AvlGraph<N, E, Ix, Mb>, node: NodeIndex<Ix>) -> Self {
        let edges = Edges::new(graph, node);
        Self { edges }
    }
}

pub struct Edges<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    graph: &'a AvlGraph<N, E, Ix, Mb>,
    stack: Vec<EdgeIndex<Ix>>,
}

impl<'a, N, E, Ix, Mb> Iterator for Edges<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
    Mb::EdgeRef: Sized,
{
    // Was: type Item = &'a Edge<E, Ix>;
    // Should be: type Item = Mb::EdgeRef<'a>;
    type Item = Mb::EdgeRef;

    fn next(&mut self) -> Option<Self::Item> {
        // Is this pop_back()????
        match self.stack.pop() {
            None => None,
            Some(idx) => {
                if idx == EdgeIndex::end() {
                    // Only hit for an empty tree.
                    return None;
                }

                let left = self.graph.edges.index(idx.index()).get_left();
                if left != EdgeIndex::end() {
                    self.stack.push(left);
                }
                let right = self.graph.edges.index(idx.index()).get_right();
                if right != EdgeIndex::end() {
                    self.stack.push(right);
                }
                Some(self.graph.edges.index(idx.index()))
            }
        }
    }
}

impl<'a, N, E, Ix, Mb> Edges<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    pub fn new(graph: &'a AvlGraph<N, E, Ix, Mb>, node: NodeIndex<Ix>) -> Self {
        let root = graph.nodes.index(node.index()).get_first_edge();
        let stack = vec![root];
        // let mut stack = LinkedList::new();
        // stack.push_back(root);
        Self { graph, stack }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use graph::avl_graph::edge::EdgeRef;
    use graph::avl_graph::node::{NodeMutRef, NodeRef};
    use graph::avl_graph::AvlGraph;
    use graph::indexing::{EdgeIndex, IndexType, NodeIndex};
    use std::convert::TryInto;
    use weight::{Weight, Weight40};

    use serde::{Deserialize, Serialize};

    #[test]
    fn test_create_graph() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        assert_eq!(graph.add_node(weight.clone()).index(), 0);
        assert_eq!(graph.add_node(weight.clone()).index(), 1);
    }

    #[test]
    fn test_add_edge() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        let q2 = graph.add_node(weight.clone());
        let q3 = graph.add_node(weight.clone());

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
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, char> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        let q2 = graph.add_node(weight.clone());

        assert_eq!(graph.add_edge(q0, q1, 'b'), Some(EdgeIndex::new(0)));
        assert_eq!(graph.add_edge(q0, q2, 'a'), Some(EdgeIndex::new(1)));
    }

    #[test]
    fn test_rotate_from_right() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());

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
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());

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
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        let q2 = graph.add_node(weight.clone());
        let q3 = graph.add_node(weight.clone());

        graph.add_balanced_edge(q1, q2, 2);
        graph.add_balanced_edge(q1, q3, 2);
        graph.add_balanced_edge(q1, q3, 3);

        graph.add_balanced_edge(q1, q3, 4);
        graph.add_balanced_edge(q1, q3, 3);
        graph.add_balanced_edge(q1, q2, 4);

        for idx in 5..16 {
            let q = graph.add_node(weight.clone());
            graph.add_balanced_edge(q1, q, idx.try_into().unwrap());
        }

        assert_eq!(graph.edge_target(q1, 2), Some(q2));
        assert_eq!(graph.edge_target(q1, 3), Some(q3));
        assert_eq!(graph.edge_target(q1, 42), None);

        assert_eq!(graph.n_edges(q1), 14);
        assert_eq!(graph.edge_tree_height(q1), 4)
    }

    #[test]
    fn test_add_balanced_edge_left_branching() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u64> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        for idx in (0..127).rev() {
            graph.add_balanced_edge(q0, q1, idx);
        }
        assert_eq!(graph.n_edges(q0), 127);
        assert_eq!(graph.edge_tree_height(q0), 7)
    }

    #[test]
    fn test_tree_construction() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());

        graph.add_balanced_edge(q1, q0, 0);
        graph.add_balanced_edge(q1, q0, 1);

        let mut root = graph.nodes[q1.index()].get_first_edge();
        let mut left: EdgeIndex = graph.edges[root.index()].left;
        let mut right: EdgeIndex = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, -1);
        assert_eq!(left, EdgeIndex::end());
        assert_eq!(graph.edges[right.index()].weight, 1);

        graph.add_balanced_edge(q1, q0, 2);

        root = graph.nodes[q1.index()].get_first_edge();
        left = graph.edges[root.index()].left;
        right = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, 0);
        assert_eq!(graph.edges[root.index()].weight, 1);
        assert_eq!(graph.edges[left.index()].weight, 0);
        assert_eq!(graph.edges[right.index()].weight, 2);
    }

    #[test]
    fn test_clone_node() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(Weight40::new(42, None, 0));
        let q1 = graph.add_node(weight.clone());
        graph.add_edge(q0, q1, 2);

        let q2 = graph.clone_node(q0);
        assert_eq!(graph.get_node(q2).get_length(), 42);
        assert_eq!(graph.edge_target(q2, 2), Some(q1));
    }

    #[test]
    fn test_reroute_edge() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        let q2 = graph.add_node(weight.clone());
        graph.add_edge(q0, q1, 2);
        assert!(graph.reroute_edge(q0, q2, 2));
        assert_eq!(graph.edge_target(q0, 2), Some(q2));
    }

    #[test]
    fn test_edges_iterator() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u32> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let q1 = graph.add_node(weight.clone());
        for idx in 0..7 {
            graph.add_balanced_edge(q0, q1, idx);
        }
        let edges: Vec<_> = graph
            .edges(q0)
            .map(|x| (x.get_weight(), x.get_target().index()))
            .collect();
        assert_eq!(
            edges,
            vec![(3, 1), (5, 1), (6, 1), (4, 1), (1, 1), (2, 1), (0, 1)]
        );

        assert_eq!(graph.balance_ratio(q0), 1.0);
        // FIXME: But stilll take the time tho
    }

    #[test]
    fn test_node_index_mut() {
        let weight = Weight40::new(0, None, 0);
        let mut graph: AvlGraph<Weight40, u32> = AvlGraph::new();
        let q0 = graph.add_node(weight.clone());
        let idx0 = NodeIndex::new(0);
        graph.get_node_mut(idx0).set_length(1);
        assert_eq!(graph.get_node(idx0).get_length(), 1);
    }
}

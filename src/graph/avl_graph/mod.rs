// Building minimal AvlGraph from the ground up.
// Support finding an edge in log(|E|) time.
// Originally based on petgraph. See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Edges stored in AVL tree: // https://stackoverflow.com/questions/7211806/how-to-implement-insertion-for-avl-tree-without-parent-pointer

use crate::comparator::Comparator;
use anyhow::Result;
use std::clone::Clone;
use std::cmp::{Eq, Ord, Ordering};
use std::path::Path;

use std::marker::PhantomData;

use crate::serde::de::DeserializeOwned;
use crate::serde::Serialize;
use std::cmp::{max, min};
use std::fmt::Debug;

use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use crate::memory_backing::{CacheConfig, DiskVec, InternallyImmutableVecBacking};
use crate::weight::Weight;

pub mod edge;
mod graph_impl;
pub mod node;
mod serde;

pub use self::edge::{AvlEdge, AvlEdgeMutRef, AvlEdgeRef};
pub use self::node::{AvlNode, AvlNodeMutRef};
use crate::graph::comparator::DEFAULT_CMP;
use crate::graph::traits::{EdgeRef, NodeRef};
use crate::memory_backing::{disk_backing, DiskBacking, MemoryBacking};
use crate::memory_backing::{RamBacking, VecBacking};

#[derive(Default)]
pub struct AvlGraph<N, E, Ix = DefaultIx, Mb = RamBacking<N, E, Ix>>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    nodes: Mb::VecN,
    edges: Mb::VecE,
    marker: PhantomData<(N, E, Ix)>,
}

impl<N, E, Ix> AvlGraph<N, E, Ix>
where
    E: Copy + Debug,
    Ix: IndexType,
    N: Weight + Clone,
{
    pub fn new() -> Self {
        let mb: RamBacking<N, E, Ix> = RamBacking::default();
        Self::new_mb(mb)
    }

    pub fn save_to_disk<P: AsRef<Path> + Clone + Debug>(&self, path: P) -> Result<()>
    where
        N: Serialize + DeserializeOwned + Default,
        E: Serialize + DeserializeOwned + Default,
        Ix: Serialize + DeserializeOwned + Default,
    {
        let mb: DiskBacking<N, E, Ix> = DiskBacking::new(path);
        let _ = DiskVec::from_vec(&self.nodes, mb.get_nodes_path());
        let _ = DiskVec::from_vec(&self.edges, mb.get_edges_path());
        Ok(())
    }
}

impl<N, E, Ix> AvlGraph<N, E, Ix, DiskBacking<N, E, Ix>>
where
    E: Copy + Debug + Serialize + DeserializeOwned + Default,
    N: Weight + Copy + Clone + Serialize + DeserializeOwned + Default,
    Ix: IndexType + Serialize + DeserializeOwned,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        let mb: DiskBacking<N, E, Ix> = DiskBacking::new(path);
        // FIXME: This can be refactored to call a method in Mb.
        let nodes =
            disk_backing::vec::Vec::load(mb.get_nodes_path(), cache_config.node_cache_size)?;
        let edges =
            disk_backing::vec::Vec::load(mb.get_edges_path(), cache_config.edge_cache_size)?;
        Ok(Self {
            nodes,
            edges,
            marker: PhantomData,
        })
    }
}

impl<N, E, Ix, Mb> AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Copy + Debug,
    Ix: IndexType,
{
    pub fn new_mb(mb: Mb) -> Self {
        let nodes = mb.new_node_vec(None, 0);
        let edges = mb.new_edge_vec(None, 0);
        AvlGraph {
            nodes,
            edges,
            marker: PhantomData,
        }
    }

    pub fn with_capacity_mb(
        mb: Mb,
        n_nodes: usize,
        n_edges: usize,
        cache_config: CacheConfig,
    ) -> Self {
        let nodes = mb.new_node_vec(Some(n_nodes), cache_config.node_cache_size);
        let edges = mb.new_edge_vec(Some(n_edges), cache_config.edge_cache_size);
        AvlGraph {
            nodes,
            edges,
            marker: PhantomData,
        }
    }
}

impl<N, E, Ix, Mb> AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Copy + Debug,
    N: Weight,
    Ix: IndexType,
{
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let node = AvlNode::new(weight);
        let node_idx = NodeIndex::new(self.nodes.len());
        assert!(<Ix as IndexType>::max_value().index() == !0 || NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    // Copy edges from a Node onto another Node
    pub fn clone_edges(&mut self, old: NodeIndex<Ix>, new: NodeIndex<Ix>) {
        let first_source_idx = self.nodes.index(old.index()).get_first_edge();
        if first_source_idx == EdgeIndex::end() {
            return;
        }

        let edge_to_clone = &self.edges.index(first_source_idx.index());
        let first_clone_edge = AvlEdge::new(edge_to_clone.get_weight(), edge_to_clone.get_target());
        let first_clone_idx = EdgeIndex::new(self.edges.len());
        self.edges.push(first_clone_edge);
        self.nodes
            .index_mut(new.index())
            .set_first_edge(first_clone_idx);
        self.clone_edges_helper(first_source_idx, first_clone_idx)
    }

    // The nodes that get passed in are the parents of the ones getting cloned.
    fn clone_edges_helper(&mut self, old: EdgeIndex<Ix>, new: EdgeIndex<Ix>) {
        if old == EdgeIndex::end() {
            return;
        }
        let left = self.edges.index(old.index()).get_left();
        let right = self.edges.index(old.index()).get_right();

        if left != EdgeIndex::end() {
            let left_weight = self.edges.index(left.index()).get_weight();
            let left_target = self.edges.index(left.index()).get_target();
            let new_left_edge = AvlEdge::new(left_weight, left_target);
            let new_left = EdgeIndex::new(self.edges.len());
            self.edges.push(new_left_edge);
            // FIXME: Handle case where
            self.edges.index_mut(new.index()).set_left(new_left);
            self.clone_edges_helper(left, new_left);
        }

        if right != EdgeIndex::end() {
            let right_weight = self.edges.index(right.index()).get_weight();
            let right_target = self.edges.index(right.index()).get_target();
            let new_right_edge = AvlEdge::new(right_weight, right_target);
            let new_right = EdgeIndex::new(self.edges.len());
            self.edges.push(new_right_edge);
            self.edges.index_mut(new.index()).set_right(new_right);
            self.clone_edges_helper(right, new_right);
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
        cmp: Box<dyn Comparator<E>>,
    ) -> (EdgeIndex<Ix>, EdgeIndex<Ix>) {
        if edge == EdgeIndex::end() {
            return (edge, last_edge);
        }

        let edge_weight = self.edges.index(edge.index()).get_weight();
        match cmp.compare(&weight, &edge_weight) {
            Ordering::Equal => (edge, last_edge),
            Ordering::Less => {
                self.binary_search(self.edges.index(edge.index()).get_left(), edge, weight, cmp)
            }
            Ordering::Greater => self.binary_search(
                self.edges.index(edge.index()).get_right(),
                edge,
                weight,
                cmp,
            ),
        }
    }

    // add_balanced_edge but for CDAWGs, where weight doesn't actually contain the token
    pub fn add_balanced_edge_cmp(
        &mut self,
        a: NodeIndex<Ix>,
        b: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) {
        let first_edge = self.get_node(a).get_first_edge();
        let new_first_edge = self.avl_insert_edge(first_edge, weight, b, cmp);
        self.get_node_mut(a).set_first_edge(new_first_edge);
    }

    fn avl_insert_edge(
        &mut self,
        root_edge_idx: EdgeIndex<Ix>,
        weight: E,
        b: NodeIndex<Ix>,
        cmp: Box<dyn Comparator<E>>,
    ) -> EdgeIndex<Ix> {
        // if we encounter null ptr, we add edge into AVL tree
        if root_edge_idx == EdgeIndex::end() {
            let edge = AvlEdge::new(weight, b);
            self.edges.push(edge);
            return EdgeIndex::new(self.edges.len() - 1);
        }

        // keep recursing into the tree according to balance tree insert rule
        let root_edge_weight = self.edges.index(root_edge_idx.index()).get_weight();

        let ordering = cmp.compare(&weight, &root_edge_weight);
        if ordering == Ordering::Less {
            let init_left_idx: EdgeIndex<Ix> = self.edges.index(root_edge_idx.index()).get_left();
            let init_balance_factor: i8 = if init_left_idx == EdgeIndex::end() {
                0
            } else {
                self.edges.index(init_left_idx.index()).get_balance_factor()
            };

            let new_left = self.avl_insert_edge(init_left_idx, weight, b, cmp);
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
        } else if ordering == Ordering::Greater {
            let init_right_idx: EdgeIndex<Ix> = self.edges.index(root_edge_idx.index()).get_right();
            let init_balance_factor: i8 = if init_right_idx == EdgeIndex::end() {
                0
            } else {
                self.edges
                    .index(init_right_idx.index())
                    .get_balance_factor()
            };

            let new_right = self.avl_insert_edge(init_right_idx, weight, b, cmp);
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

        // This is the correct edge, i.e., ordering == Ordering::Eq
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

    // get_edge_by_weight by for CDAWGs.
    pub fn get_edge_by_weight_cmp(
        &self,
        a: NodeIndex<Ix>,
        weight: E,
        cmp: Box<dyn Comparator<E>>,
    ) -> Option<EdgeIndex<Ix>> {
        let first_edge = self.get_node(a).get_first_edge();
        if first_edge == EdgeIndex::end() {
            return None;
        }
        let (e, _last_e) = self.binary_search(first_edge, EdgeIndex::end(), weight, cmp);
        if e == EdgeIndex::end() {
            return None;
        }
        Some(e)
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

    pub fn neighbors(&self, node: NodeIndex<Ix>) -> Neighbors<'_, N, E, Ix, Mb> {
        Neighbors::new(self, node)
    }

    pub fn edges(&self, edges: NodeIndex<Ix>) -> Edges<'_, N, E, Ix, Mb> {
        Edges::new(self, edges)
    }

    pub fn ordered_edges(&self, edges: NodeIndex<Ix>) -> OrderedEdges<'_, N, E, Ix, Mb> {
        OrderedEdges::new(self, edges)
    }

    // We can't use standard indexing because we have custom reference types.

    pub fn get_node(&self, node: NodeIndex<Ix>) -> Mb::NodeRef {
        self.nodes.index(node.index())
    }

    pub fn get_node_mut(&mut self, node: NodeIndex<Ix>) -> Mb::NodeMutRef {
        self.nodes.index_mut(node.index())
    }

    pub fn get_edge(&self, edge: EdgeIndex<Ix>) -> Mb::EdgeRef {
        self.edges.index(edge.index())
    }

    pub fn get_edge_mut(&mut self, edge: EdgeIndex<Ix>) -> Mb::EdgeMutRef {
        self.edges.index_mut(edge.index())
    }
}

// When there is a Comparator implicitly defined by Eq + Ord.
impl<N, E, Ix, Mb> AvlGraph<N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    E: Eq + Ord + Copy + Debug,
    N: Weight,
    Ix: IndexType,
{
    pub fn add_balanced_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) {
        self.add_balanced_edge_cmp(a, b, weight, Box::new(DEFAULT_CMP))
    }

    pub fn get_edge_by_weight(&self, a: NodeIndex<Ix>, weight: E) -> Option<EdgeIndex<Ix>> {
        self.get_edge_by_weight_cmp(a, weight, Box::new(DEFAULT_CMP))
    }

    pub fn reroute_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            return false;
        }

        let (e, _) =
            self.binary_search(first_edge, EdgeIndex::end(), weight, Box::new(DEFAULT_CMP));
        if e == EdgeIndex::end() {
            return false;
        }
        self.edges.index_mut(e.index()).set_target(b);
        true
    }

    pub fn edge_target(&self, a: NodeIndex<Ix>, weight: E) -> Option<NodeIndex<Ix>> {
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            return None;
        }

        let (e, _last_e) =
            self.binary_search(first_edge, EdgeIndex::end(), weight, Box::new(DEFAULT_CMP));
        if e == EdgeIndex::end() {
            return None;
        }
        Some(self.edges.index(e.index()).get_target())
    }

    // DONT USE THIS, here for legacy test reasons
    fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> Option<EdgeIndex<Ix>> {
        let edge = AvlEdge::new(weight, b);
        let edge_idx = EdgeIndex::new(self.edges.len());

        // look for root, simple case where no root handled
        let first_edge = self.nodes.index(a.index()).get_first_edge();
        if first_edge == EdgeIndex::end() {
            self.nodes.index_mut(a.index()).set_first_edge(edge_idx);
            self.edges.push(edge);
            return Some(edge_idx);
        }

        // binary search to find pointer where we insert new edge (edge and parent pointers)
        let (e, last_e) =
            self.binary_search(first_edge, EdgeIndex::end(), weight, Box::new(DEFAULT_CMP));
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
    }
}

pub struct Neighbors<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    edges: Edges<'a, N, E, Ix, Mb>,
}

impl<N, E, Ix, Mb> Iterator for Neighbors<'_, N, E, Ix, Mb>
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

impl<N, E, Ix, Mb> Iterator for Edges<'_, N, E, Ix, Mb>
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

pub struct OrderedEdges<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    graph: &'a AvlGraph<N, E, Ix, Mb>,
    stack: Vec<EdgeIndex<Ix>>,
}

impl<N, E, Ix, Mb> Iterator for OrderedEdges<'_, N, E, Ix, Mb>
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
                let right = self.graph.edges.index(idx.index()).get_right();
                if right != EdgeIndex::end() {
                    self.stack.push(right);
                    let mut left = self
                        .graph
                        .edges
                        .index(self.stack[self.stack.len() - 1].index())
                        .get_left();
                    while left != EdgeIndex::end() {
                        self.stack.push(left);
                        left = self
                            .graph
                            .edges
                            .index(self.stack[self.stack.len() - 1].index())
                            .get_left();
                    }
                }
                Some(self.graph.edges.index(idx.index()))
            }
        }
    }
}

impl<'a, N, E, Ix, Mb> OrderedEdges<'a, N, E, Ix, Mb>
where
    Mb: MemoryBacking<N, E, Ix>,
    Ix: IndexType,
{
    pub fn new(graph: &'a AvlGraph<N, E, Ix, Mb>, node: NodeIndex<Ix>) -> Self {
        let root = graph.nodes.index(node.index()).get_first_edge();
        let mut stack = vec![root];
        let mut left = graph.edges.index(stack[stack.len() - 1].index()).get_left();
        while left != EdgeIndex::end() {
            stack.push(left);
            left = graph.edges.index(stack[stack.len() - 1].index()).get_left();
        }

        Self { graph, stack }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
#[allow(unused_imports)]
mod tests {
    use crate::cdawg::comparator::CdawgComparator;
    use crate::graph::avl_graph::edge::AvlEdgeRef;
    use crate::graph::avl_graph::node::AvlNodeMutRef;
    use crate::graph::avl_graph::AvlGraph;
    use crate::graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
    use crate::graph::traits::{EdgeRef, NodeRef};
    use crate::weight::{DefaultWeight, Weight};
    use std::cell::RefCell;
    use std::convert::TryInto;
    use std::rc::Rc;

    #[test]
    fn test_create_graph() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        assert_eq!(graph.add_node(weight).index(), 0);
        assert_eq!(graph.add_node(weight).index(), 1);
    }

    #[test]
    fn test_rotate_from_right() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);

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
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);

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
    fn test_add_balanced_edge_cdawg_cmp() {
        let tokens = Rc::new(RefCell::new(vec![10, 11]));

        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, (DefaultIx, DefaultIx)> = AvlGraph::new();
        let source = graph.add_node(weight);
        let sink = graph.add_node(weight);

        let cmp0 = CdawgComparator::new_with_token(tokens.clone(), 10);
        graph.add_balanced_edge_cmp(
            source,
            sink,
            (DefaultIx::new(0), DefaultIx::new(2)),
            Box::new(cmp0),
        );

        let cmp1 = CdawgComparator::new_with_token(tokens.clone(), 11);
        graph.add_balanced_edge_cmp(
            source,
            sink,
            (DefaultIx::new(1), DefaultIx::new(2)),
            Box::new(cmp1),
        );
        let edge1 = graph.get_edge(graph.get_node(source).get_first_edge());
        assert_eq!(edge1.get_weight().0.index(), 0);
        assert_eq!(edge1.get_weight().1.index(), 2);
        assert_eq!(edge1.get_left(), EdgeIndex::end());
        let edge2 = graph.get_edge(edge1.get_right());
        assert_eq!(edge2.get_weight().0.index(), 1);
        assert_eq!(edge2.get_weight().1.index(), 2);
    }

    #[test]
    fn test_add_balanced_edge() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);
        let q2 = graph.add_node(weight);
        let q3 = graph.add_node(weight);

        graph.add_balanced_edge(q1, q2, 2);
        graph.add_balanced_edge(q1, q3, 2);
        graph.add_balanced_edge(q1, q3, 3);

        graph.add_balanced_edge(q1, q3, 4);
        graph.add_balanced_edge(q1, q3, 3);
        graph.add_balanced_edge(q1, q2, 4);

        for idx in 5..16 {
            let q = graph.add_node(weight);
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
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u64> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);
        for idx in (0..127).rev() {
            graph.add_balanced_edge(q0, q1, idx);
        }
        assert_eq!(graph.n_edges(q0), 127);
        assert_eq!(graph.edge_tree_height(q0), 7)
    }

    #[test]
    fn test_tree_construction() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);

        graph.add_balanced_edge(q1, q0, 0);
        graph.add_balanced_edge(q1, q0, 1);

        let mut root = graph.get_node(q1).get_first_edge();
        let mut left: EdgeIndex = graph.edges[root.index()].left;
        let mut right: EdgeIndex = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, -1);
        assert_eq!(left, EdgeIndex::end());
        assert_eq!(graph.edges[right.index()].weight, 1);

        graph.add_balanced_edge(q1, q0, 2);

        root = graph.get_node(q1).get_first_edge();
        left = graph.edges[root.index()].left;
        right = graph.edges[root.index()].right;
        assert_eq!(graph.edges[root.index()].balance_factor, 0);
        assert_eq!(graph.edges[root.index()].weight, 1);
        assert_eq!(graph.edges[left.index()].weight, 0);
        assert_eq!(graph.edges[right.index()].weight, 2);
    }

    #[test]
    fn test_clone_edges() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);
        for idx in 2..10 {
            let qi = graph.add_node(weight);
            graph.add_edge(q0, qi, idx);
        }

        graph.clone_edges(q0, q1);
        for idx in 2..10 {
            let qi: NodeIndex<DefaultIx> = NodeIndex::new(idx.into());
            assert_eq!(graph.edge_target(q1, idx), Some(qi));
        }
    }

    #[test]
    fn test_reroute_edge() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);
        let q2 = graph.add_node(weight);
        graph.add_edge(q0, q1, 2);
        assert!(graph.reroute_edge(q0, q2, 2));
        assert_eq!(graph.edge_target(q0, 2), Some(q2));
    }

    #[test]
    fn test_edges_iterator() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u32> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);
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
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u32> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let idx0 = NodeIndex::new(0);
        graph.get_node_mut(idx0).set_length(1);
        assert_eq!(graph.get_node(idx0).get_length(), 1);
    }

    #[test]
    fn test_ordered_edges() {
        let weight = DefaultWeight::new(0, None, 0);
        let mut graph: AvlGraph<DefaultWeight, u16> = AvlGraph::new();
        let q0 = graph.add_node(weight);
        let q1 = graph.add_node(weight);

        let weights: [u16; 10] = [10, 1, 11, 7, 5, 6, 9, 13, 15, 8];
        let expected: [u16; 10] = [1, 5, 6, 7, 8, 9, 10, 11, 13, 15];

        for weight in weights.iter() {
            graph.add_balanced_edge(q0, q1, *weight);
        }

        for (i, edge) in graph.ordered_edges(q0).enumerate() {
            assert_eq!(expected[i], edge.get_weight());
        }
    }
}

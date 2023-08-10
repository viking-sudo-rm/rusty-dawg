// Building minimal AvlGraph from the ground up.
// Minimize memory overhead.
// Support finding an edge in log(|E|) time.
// See https://timothy.hobbs.cz/rust-play/petgraph-internals.html
// Can also implement a version with separate Node/Edge lists and Edge pointers forming AVL tree.

// https://stackoverflow.com/questions/7211806/how-to-implement-insertion-for-avl-tree-without-parent-pointer

use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::{Eq, Ord};
use std::ops::{Index, IndexMut};

use graph::indexing::{DefaultIx, EdgeIndex, IndexType, NodeIndex};

pub mod dot;

#[derive(Deserialize, Serialize)]
pub struct AvlGraph<N, E, Ix = DefaultIx> {
    #[serde(bound(
        serialize = "N: Serialize, E: Serialize, Ix: Serialize",
        deserialize = "N: Deserialize<'de>, E: Deserialize<'de>, Ix: Deserialize<'de>",
    ))]
    nodes: Vec<Node<N, Ix>>,
    edges: Vec<Edge<E, Ix>>,
}

impl<N, E, Ix: IndexType> AvlGraph<N, E, Ix>
where
    E: Eq + Ord + Copy,
{
    pub fn new() -> Self {
        let nodes = Vec::new();
        let edges = Vec::new();
        AvlGraph { nodes, edges }
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
        match self.nodes.get_mut(a.index()) {
            Some(ptr) => ptr.weight = value,
            None => (),
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
        self.edges.get(edge.index()).map(|e| &e.weight)
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

    // insert node function (nodes in tree are edges in graph)
    // merge Pete's update before adding code
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

        // balance needs to be called here after we add the new edge
    
        // self.pre_update_balance_factors(first_edge, weight);
        // if first_edge != EdgeIndex::end() {
        //     self.balance(first_edge, EdgeIndex::end(), weight);
        // }

        // return idx
        Some(edge_idx)
        // FIXME: Implement recursive version!!!
    }

    // TODO: implement a balance factor version of this
    // pub fn add_balanced_edge(
    //     &mut self,
    //     a: NodeIndex<Ix>,
    //     b: NodeIndex<Ix>,
    //     weight: E,
    // ) -> Option<EdgeIndex<Ix>> {
    //     // node_ptr: root_ptr, x: element
    //     if node_ptr == None {
    //         node_ptr = new Node();
    //         // assign new data 
    //         return
    //     }

    //     if node_ptr.data > x {
    //         // add node_ptr.left, x
    //         self.add_balanced_edge(a, b, weight);

    //         // check if there is a balance issue
    //         if node_ptr.left.balance_factor != node_ptr.right_balance_factor {
    //             if node_ptr.left.data > x {
    //                 // outside case
    //                 node_ptr = self.rotate_from_left(node_ptr);
    //             } else {
    //                 // inside case
    //                 node_ptr = self.double_rotate_from_left(node_ptr);
    //             }
    //         }
    //     } else if node_ptr.data < x {
    //         // add node_ptr.right, x
    //         self.add_balanced_edge(a, b, weight);

    //         // check if there is a balance issue
    //         if node_ptr.left.balance_factor != node_ptr.right.balance_factor {
    //             // this condition remains same
    //             if node_ptr.left.data > x {
    //                 // outside case
    //                 node_ptr = self.rotate_from_right(node_ptr);
    //             } else {
    //                 // inside case
    //                 node_ptr = self.double_rotate_from_right(node_ptr);
    //             }
    //         }
    //     } else {
    //         raise Exception("duplicate data being inserted");
    //     }
    // }

    // Return the difference in max heights (right - left).
    // See https://en.wikipedia.org/wiki/AVL_tree
    fn pre_update_balance_factors(&mut self, e: EdgeIndex<Ix>, weight: E) {
        if e == EdgeIndex::end() {
            return;
        }

        if self.edges[e.index()].weight < weight {
            self.edges[e.index()].balance_factor -= 1;
            self.pre_update_balance_factors(self.edges[e.index()].left, weight);
            return;
        }

        if self.edges[e.index()].weight > weight {
            self.edges[e.index()].balance_factor += 1;
            self.pre_update_balance_factors(self.edges[e.index()].right, weight);
        }
    }

    // AVL tree balance insert functions
    fn rotate_from_right(
        &mut self,
        node_ptr: EdgeIndex<Ix>,
    ) -> EdgeIndex<Ix> {
        let p: EdgeIndex<Ix> = self.edges[node_ptr.index()].right;
        self.edges[node_ptr.index()].right = self.edges[p.index()].left;
        self.edges[p.index()].left = node_ptr;

        return p;
    }

    fn rotate_from_left(
        &mut self,
        node_ptr: EdgeIndex<Ix>,
    ) -> EdgeIndex<Ix> {
        let p: EdgeIndex<Ix> = self.edges[node_ptr.index()].left;
        self.edges[node_ptr.index()].left = self.edges[p.index()].right;
        self.edges[p.index()].right = node_ptr;

        return p;
    }

    fn double_rotate_from_right(
        &mut self,
        node_ptr: EdgeIndex<Ix>
    ) -> EdgeIndex<Ix> {
        self.edges[node_ptr.index()].right = self.rotate_from_left(self.edges[node_ptr.index()].right);
        return self.rotate_from_right(node_ptr);
    }

    fn double_rotate_from_left(
        &mut self,
        node_ptr: EdgeIndex<Ix>
    ) -> EdgeIndex<Ix> {
        self.edges[node_ptr.index()].left = self.rotate_from_right(self.edges[node_ptr.index()].left);
        return self.rotate_from_left(node_ptr);
    }

    // Return whether we have done a balance somewhere.
    // Implementation of: https://en.wikipedia.org/wiki/AVL_tree
    fn balance(&mut self, e: EdgeIndex<Ix>, p: EdgeIndex<Ix>, weight: E) -> bool {
        println!("  balancing {}", e.index());

        // Nothing to balance.
        if e == EdgeIndex::end() || self.edges[e.index()].weight == weight {
            return false;
        }

        // Balance everything below and return if nothing changes.
        if weight < self.edges[e.index()].weight {
            self.balance(self.edges[e.index()].left, e, weight);
        } else {
            self.balance(self.edges[e.index()].right, e, weight);
        }

        // The loop terminates at the null parent pointer.
        if p == EdgeIndex::end() {
            return true;
        }
        let new_root;

        // The right-child case.
        if e == self.edges[p.index()].right {
            if self.edges[e.index()].balance_factor > 0 {
                let r = self.edges[e.index()].right;
                if self.edges[r.index()].balance_factor < 0 {
                    // Rotate right, left.
                    println!("  rotate right/left");
                    let old_rl = self.edges[r.index()].left;
                    let old_rll = self.edges[old_rl.index()].left;
                    let old_rlr = self.edges[old_rl.index()].right;
                    self.edges[r.index()].left = old_rlr;
                    // FIXME: Correct balance factor update??
                    // https://cs.stackexchange.com/questions/16313/updating-an-avl-tree-based-on-balance-factors
                    // self.update_balance_factor(r);
                    self.edges[e.index()].right = old_rll;
                    // self.update_balance_factor(e);
                    self.edges[old_rl.index()].left = e;
                    self.edges[old_rl.index()].right = r;
                    // self.update_balance_factor(old_rl);
                    new_root = old_rl;
                } else {
                    // Rotate left.
                    println!("  rotate left");
                    let old_rl = self.edges[r.index()].left;
                    self.edges[e.index()].right = old_rl;
                    // self.update_balance_factor(e);
                    self.edges[r.index()].left = e;
                    // self.update_balance_factor(r);
                    new_root = r;
                }
            } else {
                if self.edges[e.index()].balance_factor < 0 {
                    self.edges[e.index()].balance_factor = 0;
                    return false;
                }
                self.edges[e.index()].balance_factor = 1;
                return true;
            }
        }
        // The left-child case.
        else if self.edges[e.index()].balance_factor < 0 {
            let l = self.edges[e.index()].left;
            if self.edges[l.index()].balance_factor > 0 {
                // Rotate left, right.
                println!("  rotate left/right");
                let old_lr = self.edges[l.index()].right;
                let old_lrl = self.edges[old_lr.index()].left;
                let old_lrr = self.edges[old_lr.index()].right;
                self.edges[l.index()].right = old_lrl;
                // self.update_balance_factor(l);
                self.edges[e.index()].left = old_lrr;
                // self.update_balance_factor(e);
                self.edges[old_lr.index()].left = l;
                self.edges[old_lr.index()].right = e;
                // self.update_balance_factor(old_lr);
                new_root = old_lr;
            } else {
                // Rotate right.
                println!("  rotate right");
                let old_lr = self.edges[l.index()].right;
                self.edges[e.index()].left = old_lr;
                // self.update_balance_factor(e);
                self.edges[l.index()].right = e;
                // self.update_balance_factor(l);
                new_root = l;
            }
        } else {
            if self.edges[e.index()].balance_factor > 0 {
                self.edges[e.index()].balance_factor = 0;
                return false;
            }
            self.edges[e.index()].balance_factor = -1;
            return true;
        }

        if self.edges[p.index()].weight < weight {
            self.edges[p.index()].right = new_root;
        } else {
            self.edges[p.index()].left = new_root;
        }
        println!("p.left: {}", self.edges[p.index()].left.index());
        println!("p.right: {}", self.edges[p.index()].right.index());
        true
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

    // pub fn edges(&self, a: NodeIndex<Ix>) -> Iter<'_, EdgeIndex<Ix>> {
    //     let first_index = self.nodes[a.index()].first_index;
    //     let mut stack: Vec<EdgeIndex<Ix>> = vec![first_index];

    //     while stack.len() > 0 {
    //         let e = stack.pop().unwrap();
    //         if e != EdgeIndex::end() {
    //             yield e;
    //             stack.push(self.edges[e.index()].left);
    //             stack.push(self.edges[e.index()].right);
    //         }
    //     }
    // }

    // Don't need to remove edges to build the DAWG!
    // pub fn remove_edge(&mut self, a: NodeIndex<Ix>, weight: E) -> bool {
    //     let first_edge = self.nodes[a.index()].first_edge;
    //     if first_edge == EdgeIndex::end() {
    //         return false;
    //     }

    //     let e, last_e = self.binary_search(first_edge, EdgeIndex::end(), weight);
    //     if e == EdgeIndex::end() {
    //         return false;
    //     }
    //     let left_end = self.edges[e.index()].left == EdgeIndex::end();
    //     let right_end = self.edges[e.index()].right == EdgeIndex::end();
    //     if left_end && right_end {
    //         self.update_parent_pointer(last_e, e, EdgeIndex::end());
    //     } else if left_end {
    //         let right = self.edges[e.index()].right;
    //         self.update_parent_pointer(last_e, e, right);
    //     } else if right_end {
    //         let left = self.edges[e.index()].left;
    //         self.update_parent_pointer(last_e, e, left);
    //     } else {
    //         let left = self.edges[e.index()].left;
    //         let left_left = self.edges[left.index()].left;
    //         let left_right = self.edges[left.index()].right;
    //         // TODO: balance the AVL tree?
    //     }
    //     return true;
    // }

    // pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix> {
    //     Neighbors::new(&self.nodes[a.index()])
    // }

    pub fn n_edges(&self, a: NodeIndex<Ix>) -> usize {
        let mut stack = vec![self.nodes[a.index()].first_edge];
        let mut count = 0;
        while !stack.is_empty() {
            let top = stack.pop().unwrap();
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
}

impl<N, E, Ix> Index<NodeIndex<Ix>> for AvlGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

impl<N, E, Ix> IndexMut<NodeIndex<Ix>> for AvlGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }
}

#[derive(Deserialize, Serialize)]
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

#[derive(Serialize, Deserialize)]
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

    fn height(graph: &AvlGraph<u8, u16>, e: EdgeIndex) -> usize {
        if e == EdgeIndex::end() {
            return 0;
        }
        height(graph, graph.edges[e.index()].left) + height(graph, graph.edges[e.index()].right) + 1
    }

    #[test]
    fn test_update_balance_factors() {
        let mut graph: AvlGraph<u8, u16> = AvlGraph::new();
        let q0 = graph.add_node(0);
        for idx in 1..8 {
            println!("=> height: {}", height(&graph, graph.nodes[0].first_edge));
            let qi = graph.add_node(idx);
            graph.add_edge(q0, qi, idx.into());
        }

        println!("=> height: {}", height(&graph, graph.nodes[0].first_edge));
        println!(
            "bf: {}",
            graph.edges[graph.nodes[0].first_edge.index()].balance_factor
        );
        // assert_eq!(0, 1);
        // FIXME
    }
}

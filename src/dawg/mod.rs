// See here for Graph info:
// https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
//
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
//

mod serde;

use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::cmp::{Eq, Ord};
use std::collections::LinkedList;
use std::fmt::Debug;

use graph::avl_graph::AvlGraph;
use graph::indexing::NodeIndex;
use weight::Weight;

use graph::indexing::{DefaultIx, IndexType};
use graph::memory_backing::ram_backing::RamBacking;
use graph::memory_backing::MemoryBacking;

use graph::avl_graph::edge::EdgeRef;
use graph::avl_graph::node::{NodeMutRef, NodeRef};

pub struct Dawg<E, W, Ix = DefaultIx, Mb = RamBacking<W, E, Ix>>
where
    Mb: MemoryBacking<W, E, Ix>,
    Ix: IndexType,
{
    dawg: AvlGraph<W, E, Ix, Mb>,
    initial: NodeIndex<Ix>,
}

// Currently the implementation fixes DefaultIx. Would not be too hard to generalize.
// impl<E, W, Mb> Default for Dawg<E, W, DefaultIx, Mb>
// where
//     E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug,
//     W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
//     Mb: MemoryBacking<W, E, DefaultIx>,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl<E, W> Dawg<E, W>
where
    E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug,
    W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
{
    pub fn new() -> Dawg<E, W> {
        let mb: RamBacking<W, E, DefaultIx> = RamBacking::default();
        Self::new_mb(mb)
    }

    pub fn with_capacity(n_nodes: usize, n_edges: usize) -> Dawg<E, W> {
        let mb: RamBacking<W, E, DefaultIx> = RamBacking::default();
        Self::with_capacity_mb(mb, n_nodes, n_edges)
    }
}

impl<E, W, Mb> Dawg<E, W, DefaultIx, Mb>
where
    E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug,
    W: Weight + Serialize + for<'a> Deserialize<'a> + Clone,
    Mb: MemoryBacking<W, E, DefaultIx>,
    Mb::EdgeRef: Copy,
{
    fn new_mb(mb: Mb) -> Dawg<E, W, DefaultIx, Mb> {
        let mut dawg: AvlGraph<W, E, DefaultIx, Mb> = AvlGraph::new_mb(mb);
        let initial = dawg.add_node(W::initial());
        dawg.get_node_mut(initial).increment_count();
        Dawg { dawg, initial }
    }

    fn with_capacity_mb(mb: Mb, n_nodes: usize, n_edges: usize) -> Dawg<E, W, DefaultIx, Mb> {
        let mut dawg: AvlGraph<W, E, DefaultIx, Mb> =
            AvlGraph::with_capacity_mb(mb, n_nodes, n_edges);
        let initial = dawg.add_node(W::initial());
        dawg.get_node_mut(initial).increment_count();
        Dawg { dawg, initial }
    }

    pub fn build(&mut self, text: &[E]) {
        let mut last = self.initial;
        for token in text.iter() {
            last = self.extend(*token, last);
        }
    }

    pub fn extend(&mut self, token: E, last: NodeIndex) -> NodeIndex {
        let new = self
            .dawg
            .add_node(W::extend(&self.get_node(last).get_weight()));
        // Follow failure path from last until transition is defined.
        let mut opt_state = Some(last);
        let mut opt_next_state: Option<NodeIndex> = None;
        loop {
            let q = opt_state.unwrap();
            self.dawg.add_balanced_edge(q, new, token);
            opt_state = self.get_node(q).get_failure();
            match opt_state {
                Some(state) => {
                    opt_next_state = self.transition(state, token, false);
                    if opt_next_state.is_some() {
                        break;
                    }
                }
                None => break,
            }
        }

        match opt_state {
            // There is no valid failure state for the new state.
            None => self.dawg.get_node_mut(new).set_failure(Some(self.initial)),

            // Found a failure state to fail to.
            Some(mut state) => {
                let next_state = opt_next_state.unwrap();
                if self.get_node(state).get_length() + 1 == self.get_node(next_state).get_length() {
                    // Fail to an existing state.
                    self.dawg.get_node_mut(new).set_failure(Some(next_state));
                } else {
                    // Split a state and fail to the clone of it.

                    // ==========================================
                    // Original cloning code (pre-Hackathon)
                    // ==========================================
                    let clone = self.dawg.add_node(W::split(
                        &self.get_node(state).get_weight(),
                        &self.get_node(next_state).get_weight(),
                    ));
                    let edges: Vec<_> = self
                        .dawg
                        .edges(next_state)
                        .map(|edge| (edge.get_target(), edge.get_weight()))
                        .collect();
                    for (target, weight) in edges {
                        self.dawg.add_balanced_edge(clone, target, weight);
                    }
                    // ==========================================
                    // Aug 10: First changed version to use clone
                    // let clone = self.dawg.clone_node(next_state);
                    // ==========================================
                    // Cloning logic commented out from a while ago
                    // ==========================================
                    // let weight = W::split(&self.dawg[state], &self.dawg[next_state]);
                    // let clone = self.dawg.clone_node(state);
                    // self.dawg.set_node_weight(clone, weight);
                    // ==========================================
                    self.dawg.get_node_mut(new).set_failure(Some(clone));
                    self.dawg.get_node_mut(next_state).set_failure(Some(clone));

                    // Reroute edges along failure chain.
                    let mut next_state_ = next_state;
                    loop {
                        if next_state_ == next_state {
                            self.dawg.reroute_edge(state, clone, token);
                        } else {
                            self.dawg.add_balanced_edge(state, clone, token);
                        }

                        match self.get_node(state).get_failure() {
                            None => break,
                            Some(q) => {
                                state = q;
                            }
                        }
                        if let Some(value) = self.transition(state, token, false) {
                            next_state_ = value;
                            if next_state_ != next_state {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Increment counts of all suffixes along the failure path.
        let mut opt_ptr = Some(new);
        while opt_ptr.is_some() {
            let ptr = opt_ptr.unwrap();
            self.dawg.get_node_mut(ptr).increment_count();
            opt_ptr = self.get_node(ptr).get_failure();
        }

        new
    }

    // Set the lengths field to store min factor length instead of max factor length.
    pub fn recompute_lengths(&mut self) {
        self._zero_lengths(self.initial);
        let mut queue: LinkedList<(NodeIndex, u64)> = LinkedList::new();
        queue.push_back((self.initial, 0));

        while let Some((state, length)) = queue.pop_front() {
            if self.get_node(state).get_length() != 0 {
                continue;
            }
            self.dawg.get_node_mut(state).set_length(length);
            for next_state in self.dawg.neighbors(state) {
                queue.push_back((next_state, length + 1));
            }
        }
    }

    fn _zero_lengths(&mut self, state: NodeIndex) {
        self.dawg.get_node_mut(state).set_length(0);
        // FIXME: Use Walker object here.
        let next_states: Vec<_> = self.dawg.neighbors(state).collect();
        for next_state in next_states {
            self._zero_lengths(next_state);
        }
    }

    // Compute the min factor length of this state dynamically.
    pub fn get_length(&self, mut state: NodeIndex) -> u64 {
        let mut count = 0;
        while let Some(fstate) = self.get_node(state).get_failure() {
            state = fstate;
            count += 1;
        }
        count
    }

    pub fn transition(&self, state: NodeIndex, token: E, use_failures: bool) -> Option<NodeIndex> {
        // for edge in self.dawg.edges(state) {
        //     if token == *edge.weight() {
        //         return Some(edge.target());
        //     }
        // }
        let next_state = self.dawg.edge_target(state, token);
        if next_state.is_some() {
            return next_state;
        }

        if !use_failures {
            return None;
        }
        let fail_state = self.get_node(state).get_failure();
        match fail_state {
            Some(q) => {
                // Not in the initial state.
                self.transition(q, token, use_failures)
            }
            // Only possible in the initial state.
            None => Some(self.initial),
        }
    }

    //Return the length of the largest matching suffix.
    pub fn transition_and_count(
        &self,
        state: NodeIndex,
        token: E,
        length: u64,
    ) -> (Option<NodeIndex>, u64) {
        // for edge in self.dawg.edges(state) {
        //     if token == *edge.weight() {
        //         return (Some(edge.target()), length + 1);
        //     }
        // }
        let next_state = self.dawg.edge_target(state, token);
        if next_state.is_some() {
            return (next_state, length + 1);
        }

        let fail_state = self.get_node(state).get_failure();
        match fail_state {
            Some(q) => {
                // If we fail, the length we're matching is the length of the largest suffix of the fail state.
                let new_length = self.get_node(q).get_length();
                self.transition_and_count(q, token, new_length)
            }
            // Only possible in the initial state.
            None => (Some(self.initial), 0),
        }
    }

    // Return the length of the largest substring of query that appears in the corpus.
    pub fn get_max_factor_length(&self, query: Vec<E>) -> u64 {
        let mut opt_state;
        let mut state = self.initial;
        let mut length = 0;
        let mut max_length = 0;
        for token in query {
            (opt_state, length) = self.transition_and_count(state, token, length);
            state = opt_state.unwrap();
            max_length = max(max_length, length);
        }
        max_length
    }

    // TODO: Can build full substring vector for query.

    pub fn get_node(&self, state: NodeIndex) -> Mb::NodeRef {
        self.dawg.get_node(state)
    }

    pub fn get_initial(&self) -> NodeIndex {
        self.initial
    }

    pub fn node_count(&self) -> usize {
        self.dawg.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.dawg.edge_count()
    }

    pub fn balance_ratio(&self, n_states: usize) -> f64 {
        let mut max_ratio = 1.;
        for _state in 0..n_states {
            let ratio = self.dawg.balance_ratio(self.get_initial());
            if ratio > max_ratio {
                max_ratio = ratio;
            }
        }
        max_ratio
    }

    pub fn get_graph(&self) -> &AvlGraph<W, E, DefaultIx, Mb> {
        &self.dawg
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use dawg::Dawg;
    use weight::Weight;

    use graph::avl_graph::node::NodeRef;
    use graph::indexing::NodeIndex;

    use bincode::{deserialize_from, serialize_into};
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use tempfile::NamedTempFile;
    use weight::weight40::DefaultWeight;

    #[test]
    fn test_build_bab() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['b', 'a', 'b']);

        let q0 = NodeIndex::new(0);
        let q1 = NodeIndex::new(1);
        let q2 = NodeIndex::new(2);
        let q3 = NodeIndex::new(3);

        assert_eq!(dawg.dawg.edge_target(q0, 'b'), Some(q1));
        assert_eq!(dawg.dawg.edge_target(q0, 'a'), Some(q2));
        assert_eq!(dawg.dawg.edge_target(q1, 'a'), Some(q2));
        assert_eq!(dawg.dawg.edge_target(q2, 'b'), Some(q3));

        assert_eq!(dawg.dawg.get_node(q0).get_failure(), None);
        assert_eq!(dawg.dawg.get_node(q1).get_failure(), Some(q0));
        assert_eq!(dawg.dawg.get_node(q2).get_failure(), Some(q0));
        assert_eq!(dawg.dawg.get_node(q3).get_failure(), Some(q1));

        assert_eq!(dawg.dawg.get_node(q0).get_count(), 4);
        assert_eq!(dawg.dawg.get_node(q1).get_count(), 2);
        assert_eq!(dawg.dawg.get_node(q2).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(q3).get_count(), 1);
    }

    #[test]
    fn test_build_abcab() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b', 'c', 'a', 'b']);
        dawg.recompute_lengths();
        assert_eq!(dawg.get_max_factor_length("ab".chars().collect()), 2);
        assert_eq!(dawg.get_max_factor_length("abc".chars().collect()), 3);
        assert_eq!(dawg.get_max_factor_length("ca".chars().collect()), 2);
        assert_eq!(dawg.get_max_factor_length("z".chars().collect()), 0);
        assert_eq!(dawg.get_max_factor_length("zzbcazz".chars().collect()), 3);

        // println!("{:?}", Dot::new(dawg.get_graph()));

        assert_eq!(dawg.dawg.get_node(NodeIndex::new(0)).get_count(), 6);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(1)).get_count(), 2);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(2)).get_count(), 2);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(3)).get_count(), 1);
    }

    #[test]
    fn test_build_abb() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b', 'b']);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(0)).get_count(), 4);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(1)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(2)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(3)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(4)).get_count(), 2);
    }

    #[test]
    fn test_build_brown() {
        let corpus = "Communication
        may be facilitated by means of the high visibility within the larger
        community. Intense interaction is easier where segregated living and
        occupational segregation mark off a group from the rest of the community,
        as in the case of this population. However, the factor of physical  
        isolation is not a static situation. Although the Brandywine population
        is still predominantly rural, there are indications of a consistent
        and a statistically significant trend away from the older and
        relatively isolated rural communities **h urbanization appears to be";
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        println!("Start build!");
        let chars: Vec<char> = corpus.chars().collect();
        dawg.build(&chars);
        // FIXME
        // dawg.recompute_lengths();
        // assert_eq!(dawg.get_max_factor_length("How".chars().collect()), 3);
        // assert_eq!(dawg.get_max_factor_length("However,".chars().collect()), 8);
        // assert_eq!(
        //     dawg.get_max_factor_length("static~However, the farce".chars().collect()),
        //     15
        // );
        // assert_eq!(
        //     dawg.get_max_factor_length("However, the zzz".chars().collect()),
        //     13
        // );
    }

    #[test]
    fn test_get_length() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b']);
        let state = NodeIndex::new(2);
        assert_eq!(dawg.dawg.get_node(state).get_length(), 2);
        assert_eq!(dawg.get_length(state), 1);
        dawg.recompute_lengths();
        assert_eq!(dawg.dawg.get_node(state).get_length(), 1);
    }

    #[test]
    fn test_serialize_deserialize_to_string() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b', 'c', 'd']);

        let encoded: Vec<u8> = bincode::serialize(&dawg).unwrap();
        let decoded: Dawg<char, DefaultWeight> = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(decoded.node_count(), 5);
    }

    #[test]
    fn test_serialize_deserialize_to_file() {
        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&['a', 'b', 'c', 'd']);

        let mut file = NamedTempFile::new().expect("Failed to create file");
        serialize_into(&file, &dawg).expect("Failed to serialize");
        file.seek(SeekFrom::Start(0)).expect(""); // Need to go to beginning of file.
        let decoded: Dawg<char, DefaultWeight> =
            deserialize_from(&file).expect("Failed to deserialize");
        assert_eq!(decoded.node_count(), 5);
    }
}

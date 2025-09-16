// Implementation of the vanilla DAWG
//
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
//

mod serde;

use crate::serde::{Deserialize, Serialize};
use anyhow::Result;
use std::cmp::max;
use std::cmp::{Eq, Ord};
use std::collections::LinkedList;
use std::fmt::Debug;
use std::path::Path;

use crate::graph::avl_graph::AvlGraph;
use crate::graph::indexing::NodeIndex;
use crate::weight::{DefaultWeight, Weight};

use crate::graph::indexing::{DefaultIx, IndexType};
use crate::memory_backing::{CacheConfig, DiskBacking, MemoryBacking, RamBacking};
use crate::serde::de::DeserializeOwned; // The global serde, not the submodule

use crate::graph::avl_graph::node::AvlNodeMutRef;
use crate::graph::traits::NodeRef;

pub struct Dawg<E, W, Ix = DefaultIx, Mb = RamBacking<W, E, Ix>>
where
    Mb: MemoryBacking<W, E, Ix>,
    Ix: IndexType,
{
    dawg: AvlGraph<W, E, Ix, Mb>,
    initial: NodeIndex<Ix>,
    max_length: Option<u64>,
}

impl<E, W> Dawg<E, W>
where
    E: Eq + Ord + Serialize + for<'de> Deserialize<'de> + Copy + Debug,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new() -> Self {
        let mb: RamBacking<W, E, DefaultIx> = RamBacking::default();
        Self::new_mb(mb, None)
    }
}

impl<E, W> Default for Dawg<E, W>
where
    E: Eq + Ord + Serialize + for<'de> Deserialize<'de> + Copy + Debug,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E, W> Dawg<E, W, DefaultIx, DiskBacking<W, E, DefaultIx>>
where
    E: Eq + Ord + Copy + Debug + Serialize + DeserializeOwned + Default,
    W: Weight + Copy + Clone + Serialize + DeserializeOwned + Default,
{
    pub fn load<P: AsRef<Path> + Clone + std::fmt::Debug>(
        path: P,
        cache_config: CacheConfig,
    ) -> Result<Self> {
        let dawg = AvlGraph::load(path, cache_config)?;
        Ok(Self {
            dawg,
            initial: NodeIndex::new(0), // FIXME: Assumes that the initial state was numbered as 0.
            max_length: None, // FIXME: Doesn't matter after building, but could load from config.
        })
    }
}

impl<E, W, Mb> Dawg<E, W, DefaultIx, Mb>
where
    E: Eq + Ord + Serialize + for<'de> Deserialize<'de> + Copy + Debug,
    W: Weight + Serialize + for<'de> Deserialize<'de> + Clone,
    Mb: MemoryBacking<W, E, DefaultIx>,
    Mb::EdgeRef: Copy,
{
    pub fn new_mb(mb: Mb, max_length: Option<u64>) -> Dawg<E, W, DefaultIx, Mb> {
        let mut dawg: AvlGraph<W, E, DefaultIx, Mb> = AvlGraph::new_mb(mb);
        let initial = dawg.add_node(W::initial());
        dawg.get_node_mut(initial).increment_count();
        Dawg {
            dawg,
            initial,
            max_length,
        }
    }

    pub fn with_capacity_mb(
        mb: Mb,
        max_length: Option<u64>,
        n_nodes: usize,
        n_edges: usize,
        cache_config: CacheConfig,
    ) -> Dawg<E, W, DefaultIx, Mb> {
        let mut dawg: AvlGraph<W, E, DefaultIx, Mb> =
            AvlGraph::with_capacity_mb(mb, n_nodes, n_edges, cache_config);
        let initial = dawg.add_node(W::initial());
        dawg.get_node_mut(initial).increment_count();
        Dawg {
            dawg,
            initial,
            max_length,
        }
    }

    pub fn build(&mut self, text: &[E]) {
        let mut last = self.initial;
        let mut length = 0;
        for token in text.iter() {
            (last, length) = self.extend(*token, last, length);
        }
    }

    pub fn extend(&mut self, token: E, mut last: NodeIndex, mut length: u64) -> (NodeIndex, u64) {
        // If we hit maximum length, fail once, then extend (doesn't need to be recursive!)
        if self.max_length.is_some() && (length == self.max_length.unwrap()) {
            if let Some(phi) = self.get_node(last).get_failure() {
                last = phi;
                length = self.get_node(phi).get_length();
            }
        }

        // With max length or multiple documents, the transition sometimes already exists.
        let next_new = self.transition(last, token, false);
        if let Some(next_q) = next_new {
            self.dawg.get_node_mut(next_q).increment_count();
            return (next_q, length + 1);
        }

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
                    let clone = self.dawg.add_node(W::split(
                        &self.get_node(state).get_weight(),
                        &self.get_node(next_state).get_weight(),
                    ));
                    // let edges: Vec<_> = self
                    //     .dawg
                    //     .edges(next_state)
                    //     .map(|edge| (edge.get_target(), edge.get_weight()))
                    //     .collect();
                    // for (target, weight) in edges {
                    //     self.dawg.add_balanced_edge(clone, target, weight);
                    // }
                    self.dawg.clone_edges(next_state, clone);
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

        (new, length + 1)
    }

    pub fn end_document(
        &mut self,
        mut last: NodeIndex,
        doc_id_token: E,
        doc_id: u64,
    ) -> (NodeIndex, u64) {
        loop {
            match self.transition(last, doc_id_token, false) {
                Some(doc_state) => {
                    last = doc_state;
                }
                None => {
                    // Add a special node representing the end of a document.
                    let dnode = self.dawg.add_node(W::new(doc_id, None, 0));
                    self.dawg.add_balanced_edge(last, dnode, doc_id_token);
                    break;
                }
            }
        }
        (self.get_initial(), 0)
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

// pyo3 requires that types implement Send
unsafe impl<Mb> Send for Dawg<u16, DefaultWeight, DefaultIx, Mb> where
    Mb: MemoryBacking<DefaultWeight, u16, DefaultIx>
{
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use bincode::{deserialize_from, serialize_into};
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use tempfile::tempdir;
    use tempfile::NamedTempFile;

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

    #[test]
    fn test_build_abb_on_disk() {
        let tmp_dir = tempdir().unwrap();
        type Mb = DiskBacking<DefaultWeight, char, DefaultIx>;
        let mb: Mb = DiskBacking::new(tmp_dir.path());
        let mut dawg: Dawg<char, DefaultWeight, DefaultIx, Mb> = Dawg::new_mb(mb, None);
        dawg.build(&['a', 'b', 'b']);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(0)).get_count(), 4);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(1)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(2)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(3)).get_count(), 1);
        assert_eq!(dawg.dawg.get_node(NodeIndex::new(4)).get_count(), 2);
    }

    #[test]
    fn test_build_brown_ram_disk() {
        let corpus = "Communication
        may be facilitated by means of the high visibility within the larger
        community. Intense interaction is easier where segregated living and
        occupational segregation mark off a group from the rest of the community,
        as in the case of this population. However, the factor of physical  
        isolation is not a static situation. Although the Brandywine population
        is still predominantly rural, there are indications of a consistent
        and a statistically significant trend away from the older and
        relatively isolated rural communities **h urbanization appears to be";
        let chars: Vec<char> = corpus.chars().collect();

        let test = "stat trend";
        let test_chars: Vec<char> = test.chars().collect();

        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        dawg.build(&chars);

        let tmp_dir = tempdir().unwrap();
        type Mb = DiskBacking<DefaultWeight, char, DefaultIx>;
        let mb: Mb = DiskBacking::new(tmp_dir.path());
        let mut disk_dawg: Dawg<char, DefaultWeight, DefaultIx, Mb> = Dawg::new_mb(mb, None);
        disk_dawg.build(&chars);

        let mut dawg_state = dawg.get_initial();
        let mut disk_state = disk_dawg.get_initial();
        for token in test_chars {
            dawg_state = dawg.transition(dawg_state, token, true).unwrap();
            disk_state = disk_dawg.transition(disk_state, token, true).unwrap();
            assert_eq!(dawg_state, disk_state);
        }
    }

    #[test]
    fn test_build_brown_max_length() {
        let corpus = "Communication
        may be facilitated by means of the high visibility within the larger
        community. Intense interaction is easier where segregated living and
        occupational segregation mark off a group from the rest of the community,
        as in the case of this population. However, the factor of physical  
        isolation is not a static situation. Although the Brandywine population
        is still predominantly rural, there are indications of a consistent
        and a statistically significant trend away from the older and
        relatively isolated rural communities **h urbanization appears to be";
        let chars: Vec<char> = corpus.chars().collect();

        type Mb = RamBacking<DefaultWeight, char, DefaultIx>;
        let mut dawg1: Dawg<char, DefaultWeight> = Dawg::new_mb(Mb::default(), None);
        dawg1.build(&chars);
        let mut dawg2: Dawg<char, DefaultWeight> = Dawg::new_mb(Mb::default(), Some(50));
        dawg2.build(&chars);

        let query: &str = "stat trend predom rural";
        let mut state1 = Some(dawg1.get_initial());
        let mut state2 = Some(dawg2.get_initial());
        let mut length1 = 0;
        let mut length2 = 0;
        for token in query.chars() {
            (state1, length1) = dawg1.transition_and_count(state1.unwrap(), token, length1);
            (state2, length2) = dawg2.transition_and_count(state2.unwrap(), token, length2);
            assert_eq!(length1, length2);
        }

        // We run into issues at 45 here, even though length is only 33
        let query2 = "olation is not a static situation";
        println!("query length: {}", query2.len());
        state1 = Some(dawg1.get_initial());
        state2 = Some(dawg2.get_initial());
        length1 = 0;
        length2 = 0;
        for token in query2.chars() {
            (state1, length1) = dawg1.transition_and_count(state1.unwrap(), token, length1);
            (state2, length2) = dawg2.transition_and_count(state2.unwrap(), token, length2);
            println!("length1: {}, length2: {}", length1, length2);
            assert_eq!(length1, length2);
        }
    }

    #[test]
    pub fn test_multiple_docs() {
        let docs: Vec<&str> = vec!["abb", "aca"];
        let doc_id_token = '$';

        let mut dawg: Dawg<char, DefaultWeight> = Dawg::new();
        let mut last = dawg.get_initial();
        let mut length = 0;
        for (doc_id, doc) in docs.iter().enumerate() {
            for token in doc.chars() {
                (last, length) = dawg.extend(token, last, length);
            }
            (last, length) = dawg.end_document(last, doc_id_token, doc_id.try_into().unwrap());
        }

        // Shared prefix.
        let q0 = dawg.get_initial();
        let q1 = dawg.transition(q0, 'a', false).unwrap();
        assert_eq!(dawg.get_node(q1).get_length(), 1);
        assert_eq!(dawg.get_node(q1).get_count(), 3);

        // Branch of abb.
        let q2_abb = dawg.transition(q1, 'b', false).unwrap();
        assert_eq!(dawg.get_node(q2_abb).get_length(), 2);
        assert_eq!(dawg.get_node(q2_abb).get_count(), 1);
        let q3_abb = dawg.transition(q2_abb, 'b', false).unwrap();
        assert_eq!(dawg.get_node(q3_abb).get_length(), 3);
        assert_eq!(dawg.get_node(q3_abb).get_count(), 1);
        let doc_abb = dawg.transition(q3_abb, doc_id_token, false).unwrap();
        assert_eq!(dawg.get_node(doc_abb).get_length(), 0); // Document ID 0

        // Branch of aca.
        let q2_aca = dawg.transition(q1, 'c', false).unwrap();
        assert_eq!(dawg.get_node(q2_aca).get_length(), 2);
        assert_eq!(dawg.get_node(q2_aca).get_count(), 1);
        assert_ne!(q2_abb, q2_aca);
        let q3_aca = dawg.transition(q2_aca, 'a', false).unwrap();
        assert_eq!(dawg.get_node(q3_aca).get_length(), 3);
        assert_eq!(dawg.get_node(q3_aca).get_count(), 1);
        assert_ne!(q3_abb, q3_aca);
        let doc_aca = dawg.transition(q3_aca, doc_id_token, false).unwrap();
        assert_eq!(dawg.get_node(doc_aca).get_length(), 1); // Document ID 1

        assert_eq!(dawg.transition(q1, 'a', false), None);
        assert_eq!(dawg.transition(q2_abb, 'a', false), None);
        assert_eq!(dawg.transition(q2_aca, 'b', false), None);
    }
}

// See here for Graph info:
// https://docs.rs/petgraph/latest/petgraph/graph/struct.Graph.html
// 
// See here for Suffix Automaton algorithm in Python:
// https://github.com/viking-sudo-rm/knn-transformers/blob/master/src/suffix_dfa_builder.py
// 

use std::cmp::max;
use std::cmp::{Eq, Ord};
use std::fmt::Debug;
use std::collections::LinkedList;
use serde::{Serialize, Deserialize};

// use vec_graph::dot::Dot;
use weight::{Weight40, Weight};
use graph::indexing::NodeIndex;
use graph::vec_graph::Graph;

#[derive(Serialize, Deserialize)]
pub struct Dawg<E>
where E: Eq + Copy + Debug {
    #[serde(bound(
        serialize = "E: Serialize",
        deserialize = "E: Deserialize<'de>",
    ))]
    dawg: Graph<Weight40, E>,
    initial: NodeIndex,
}

// TODO: Make weight type generic with default of Weight40
impl<E> Dawg<E>
where E: Eq + Ord + Serialize + for<'a> Deserialize<'a> + Copy + Debug {

    pub fn new() -> Dawg<E> {
        let mut dawg = Graph::<Weight40, E>::new();
        let initial = dawg.add_node(Weight40::initial());
        dawg[initial].increment_count();
        Dawg {dawg: dawg, initial: initial}
    }

    pub fn with_capacity(n_nodes: usize) -> Dawg<E> {
        let mut dawg = Graph::<Weight40, E>::with_capacity(n_nodes);
        let initial = dawg.add_node(Weight40::initial());
        dawg[initial].increment_count();
        Dawg {dawg: dawg, initial: initial}
    }

    pub fn build(&mut self, text: &Vec<E>) {
        let mut last = self.initial;
        for token in text.iter() {
            last = self.extend(*token, last);
        }
    }

    pub fn extend(&mut self, token: E, last: NodeIndex) -> NodeIndex {
        let new = self.dawg.add_node(Weight40::extend(&self.dawg[last]));
        // Follow failure path from last until transition is defined.
        let mut opt_state = Some(last);
        let mut opt_next_state: Option<NodeIndex> = None;
        loop {
            let q = opt_state.unwrap();
            self.dawg.add_edge(q, new, token);
            opt_state = self.dawg[q].get_failure();
            match opt_state {
                Some(state) => {
                    opt_next_state = self.transition(state, token, false);
                    if opt_next_state.is_some() {
                        break;
                    }
                },
                None => break,
            }
        }
    
        match opt_state {
            // There is no valid failure state for the new state.
            None => (&mut self.dawg[new]).set_failure(Some(self.initial)),
    
            // Found a failure state to fail to.
            Some(mut state) => {
                let next_state = opt_next_state.unwrap();
                if self.dawg[state].get_length() + 1 == self.dawg[next_state].get_length() {
                    // Fail to an existing state.
                    (&mut self.dawg[new]).set_failure(Some(next_state));
                }
                
                else {
                    // Split a state and fail to the clone of it.
                    let clone = self.dawg.add_node(Weight40::split(&self.dawg[state], &self.dawg[next_state]));
                    let edges: Vec<_> = self.dawg.edges(next_state).map(|edge| (edge.target(), *edge.weight())).collect();
                    for (target, weight) in edges {
                        self.dawg.add_edge(clone, target, weight);
                    }
                    // FIXME: For some reason, calling clone has infinite loop/hangs. Theoretically, it should be better??
                    // let weight = Weight40::split(&self.dawg[state], &self.dawg[next_state]);
                    // let clone = self.dawg.clone_node(state);
                    // self.dawg.set_node_weight(clone, weight);
                    (&mut self.dawg[new]).set_failure(Some(clone));
                    (&mut self.dawg[next_state]).set_failure(Some(clone));
    
                    // Reroute edges along failure chain.
                    let mut next_state_ = next_state;
                    loop {
                        if next_state_ == next_state {
                            self.dawg.reroute_edge(state, clone, token);
                        } else {
                            self.dawg.add_edge(state, clone, token);
                        }
    
                        match self.dawg[state].get_failure() {
                            None => break,
                            Some(q) => {
                                state = q;
                            },
                        }
                        match self.transition(state, token, false) {
                            Some(value) => {
                                next_state_ = value;
                                if next_state_ != next_state {
                                    break;
                                }
                            },
                            None => {},
                        }
                    }
                }
            },
        }

        // Increment counts of all suffixes along the failure path.
        let mut opt_ptr = Some(new);
        while opt_ptr.is_some() {
            let ptr = opt_ptr.unwrap();
            (&mut self.dawg[ptr]).increment_count();
            opt_ptr = self.dawg[ptr].get_failure();
        }

        return new;
    }

    // Set the lengths field to store min factor length instead of max factor length.
    pub fn recompute_lengths(&mut self) {
        self._zero_lengths(self.initial);
        let mut queue: LinkedList<(NodeIndex, u64)> = LinkedList::new();
        queue.push_back((self.initial, 0));

        loop {
            match queue.pop_front() {
                Some((state, length)) => {
                    if self.dawg[state].get_length() != 0 {
                        continue;
                    }
                    (&mut self.dawg[state]).set_length(length);
                    for next_state in self.dawg.neighbors(state) {
                        queue.push_back((next_state, length + 1));
                    }
                },
                None => break
            }
        }
    }

    fn _zero_lengths(&mut self, state: NodeIndex) {
        (&mut self.dawg[state]).set_length(0);
        // FIXME: Use Walker object here.
        let next_states: Vec<_> = self.dawg.neighbors(state).collect();
        for next_state in next_states {
            self._zero_lengths(next_state);
        }
    }

    // Compute the min factor length of this state dynamically.
    pub fn get_length(&self, mut state: NodeIndex) -> u64 {
        let mut count = 0;
        loop {
            match self.dawg[state].get_failure() {
                Some(fstate) => {
                    state = fstate;
                    count += 1;
                },
                None => {break},
            }
        }
        count
    }

    pub fn transition(&self, state: NodeIndex, token: E, use_failures: bool) -> Option<NodeIndex> {
        for edge in self.dawg.edges(state) {
            if token == *edge.weight() {
                return Some(edge.target());
            }
        }
        // let next_state = self.dawg.edge_target(state, token);
        // if next_state.is_some() {
        //     println!("{}, {:?} -> {}", state.index(), token, next_state.unwrap().index());
        //     return next_state;
        // }
    
        if !use_failures {
            return None;
        }
        let fail_state = self.dawg[state].get_failure();
        match fail_state {
            Some(q) => {
                // Not in the initial state.
                return self.transition(q, token, use_failures);
            },
            // Only possible in the initial state.
            None => return Some(self.initial),
        }    
    }

    //Return the length of the largest matching suffix.
    pub fn transition_and_count(&self, state: NodeIndex, token: E, length: u64) -> (Option<NodeIndex>, u64) {
        // for edge in self.dawg.edges(state) {
        //     if token == *edge.weight() {
        //         return (Some(edge.target()), length + 1);
        //     }
        // }
        let next_state = self.dawg.edge_target(state, token);
        if next_state.is_some() {
            return (next_state, length + 1);
        }
    
        let fail_state = self.dawg[state].get_failure();
        match fail_state {
            Some(q) => {
                // If we fail, the length we're matching is the length of the largest suffix of the fail state.
                let new_length = self.dawg[q].get_length();
                return self.transition_and_count(q, token, new_length);
            },
            // Only possible in the initial state.
            None => return (Some(self.initial), 0),
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
        return max_length;
    }

    // TODO: Can build full substring vector for query.

    pub fn get_weight(&self, state: NodeIndex) -> &Weight40 {
        &self.dawg[state]
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

    pub fn get_graph(&self) -> &Graph<Weight40, E> {
        &self.dawg
    }

}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use dawg::Dawg;
    use weight::Weight;

    use graph::vec_graph::dot::Dot;
    use graph::indexing::NodeIndex;

    use std::fs::File;
    use std::io::{Read, Write, Seek, SeekFrom};
    use bincode::{serialize_into, deserialize_from};
    use tempfile::NamedTempFile;

    #[test]
    fn test_build_bab() {
        let mut dawg = Dawg::new();
        dawg.build(&"bab".chars().collect());

        let q0 = NodeIndex::new(0);
        let q1 = NodeIndex::new(1);
        let q2 = NodeIndex::new(2);
        let q3 = NodeIndex::new(3);

        assert_eq!(dawg.dawg.edge_target(q0, 'b'), Some(q1));
        assert_eq!(dawg.dawg.edge_target(q0, 'a'), Some(q2));
        assert_eq!(dawg.dawg.edge_target(q1, 'a'), Some(q2));
        assert_eq!(dawg.dawg.edge_target(q2, 'b'), Some(q3));

        assert_eq!(dawg.dawg[q0].get_failure(), None);
        assert_eq!(dawg.dawg[q1].get_failure(), Some(q0));
        assert_eq!(dawg.dawg[q2].get_failure(), Some(q0));
        assert_eq!(dawg.dawg[q3].get_failure(), Some(q1));

        assert_eq!(dawg.dawg[q0].get_count(), 4);
        assert_eq!(dawg.dawg[q1].get_count(), 2);
        assert_eq!(dawg.dawg[q2].get_count(), 1);
        assert_eq!(dawg.dawg[q3].get_count(), 1);
    }

    #[test]
    fn test_build_abcab() {
        let mut dawg = Dawg::new();
        dawg.build(&"abcab".chars().collect());
        dawg.recompute_lengths();
        assert_eq!(dawg.get_max_factor_length("ab".chars().collect()), 2);
        assert_eq!(dawg.get_max_factor_length("abc".chars().collect()), 3);
        assert_eq!(dawg.get_max_factor_length("ca".chars().collect()), 2);
        assert_eq!(dawg.get_max_factor_length("z".chars().collect()), 0);
        assert_eq!(dawg.get_max_factor_length("zzbcazz".chars().collect()), 3);

        // println!("{:?}", Dot::new(dawg.get_graph()));

        assert_eq!(dawg.dawg[NodeIndex::new(0)].get_count(), 6);
        assert_eq!(dawg.dawg[NodeIndex::new(1)].get_count(), 2);
        assert_eq!(dawg.dawg[NodeIndex::new(2)].get_count(), 2);
        assert_eq!(dawg.dawg[NodeIndex::new(3)].get_count(), 1);
    }

    #[test]
    fn test_build_abb() {
        let mut dawg = Dawg::new();
        dawg.build(&"abb".chars().collect());
        assert_eq!(dawg.dawg[NodeIndex::new(0)].get_count(), 4);
        assert_eq!(dawg.dawg[NodeIndex::new(1)].get_count(), 1);
        assert_eq!(dawg.dawg[NodeIndex::new(2)].get_count(), 1);
        assert_eq!(dawg.dawg[NodeIndex::new(3)].get_count(), 1);
        assert_eq!(dawg.dawg[NodeIndex::new(4)].get_count(), 2);
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
        let mut dawg = Dawg::new();
        println!("Start build!");
        dawg.build(&corpus.chars().collect());
        dawg.recompute_lengths();
        assert_eq!(dawg.get_max_factor_length("How".chars().collect()), 3);
        assert_eq!(dawg.get_max_factor_length("However,".chars().collect()), 8);
        assert_eq!(dawg.get_max_factor_length("static~However, the farce".chars().collect()), 15);
        assert_eq!(dawg.get_max_factor_length("However, the zzz".chars().collect()), 13);
    }

    #[test]
    fn test_get_length() {
        let mut dawg = Dawg::new();
        dawg.build(&"ab".chars().collect());
        let state = NodeIndex::new(2);
        assert_eq!(dawg.dawg[state].get_length(), 2);
        assert_eq!(dawg.get_length(state), 1);
        dawg.recompute_lengths();
        assert_eq!(dawg.dawg[state].get_length(), 1);
    }

    #[test]
    fn test_serialize_deserialize_to_string() {
        let mut dawg = Dawg::new();
        dawg.build(&"abcd".chars().collect());
    
        let encoded: Vec<u8> = bincode::serialize(&dawg).unwrap();
        let decoded: Dawg<char> = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(decoded.node_count(), 5);
    }

    #[test]
    fn test_serialize_deserialize_to_file() {
        let mut dawg = Dawg::new();
        dawg.build(&"abcd".chars().collect());

        let mut file = NamedTempFile::new().expect("Failed to create file");
        serialize_into(&file, &dawg).expect("Failed to serialize");
        file.seek(SeekFrom::Start(0)).expect("");  // Need to go to beginning of file.
        let decoded: Dawg<char> = deserialize_from(&file).expect("Failed to deserialize");
        assert_eq!(decoded.node_count(), 5);
    }

}
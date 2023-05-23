use bitvec::prelude::*;

use petgraph::graph::NodeIndex;

use dawg::Dawg;

pub struct InverseFailuresMap {
    map: Vec<Vec<NodeIndex>>,
    visited: BitVec<u8, Msb0>,
}

impl InverseFailuresMap {

    pub fn new(n_states: usize) -> Self {
        let mut map = Vec::new();
        for _ in 0..n_states {
            map.push(Vec::new());
        }
        let visited = bitvec![u8, Msb0; 0; n_states];
        Self {map, visited}
    }

    pub fn clear(&mut self) {
        for subvec in &mut self.map {
            subvec.clear();
        }
        self.visited.fill(false);
    }

    pub fn build(&mut self, dawg: &Dawg) {
        let mut stack = vec![dawg.get_initial()];
        loop {
            match stack.pop() {
                Some(state) => {
                    if self.visited[state.index()] {
                        continue;
                    }
                    let weight = dawg.get_weight(state);
                    match weight.get_failure() {
                        Some(fail_state) => {
                            self.map[fail_state.index()].push(state);
                        }
                        None => {}
                    }
                    self.visited.set(state.index(), true);
                    // Reverse the iterator here? Shouldn't matter.
                    for next_state in dawg.get_graph().neighbors(state) {
                        stack.push(next_state);
                    }
                }
                None => {break}
            }
        }
    }

    // TODO: Could instead build these online?
    pub fn get_inverse_failures(&self, state: NodeIndex) -> &Vec<NodeIndex> {
        return &self.map[state.index()];
    }

    pub fn compute_counts(&self, dawg: &Dawg, counts: &mut Vec<usize>) {
        self._compute_counts(dawg, counts, dawg.get_initial());
    }

    // TODO: Could make not recursive.
    pub fn _compute_counts(&self, dawg: &Dawg, counts: &mut Vec<usize>, state: NodeIndex) {
        for next_state in self.get_inverse_failures(state) {
            self._compute_counts(dawg, counts, *next_state);
        }

        // println!("state: {:?}", state);
        let mut count = 0;
        if dawg.get_weight(state).is_solid() {
            // println!("+1 from solid");
            count += 1;
        }
        for next_state in self.get_inverse_failures(state) {
            count += counts[next_state.index()];
            // println!("+{} from {}", counts[next_state.index()], next_state.index());
        }
        counts[state.index()] = count;
    }

}

#[cfg(test)]
mod tests {

    use inverse_failures::InverseFailuresMap;
    use Dawg;
    use NodeIndex;

    use petgraph::dot::Dot;

    #[test]
    fn test_inverse_failures() {
        let i0 = NodeIndex::new(0);
        let i1 = NodeIndex::new(1);
        let i4 = NodeIndex::new(4);

        let mut dawg = Dawg::new();
        dawg.build("abb");

        let mut map = InverseFailuresMap::new(dawg.node_count());
        map.build(&dawg);
        assert_eq!(*map.get_inverse_failures(i0), vec![i1, i4]);
    }

    #[test]
    fn test_compute_counts() {
        let mut dawg = Dawg::new();
        dawg.build("abb");
        let mut map = InverseFailuresMap::new(dawg.node_count());
        map.build(&dawg);
        let mut counts = vec![0; dawg.node_count()];
        map.compute_counts(&dawg, &mut counts);
        assert_eq!(counts, vec![4, 1, 1, 1, 2]);
    }

    #[test]
    fn test_compute_counts_incremental() {
        let mut dawg = Dawg::new();
        let mut map = InverseFailuresMap::new(5);
        let mut counts = vec![0; 5];

        let mut last = dawg.get_initial();
        last = dawg.extend('a', last);
        map.build(&dawg);
        map.compute_counts(&dawg, &mut counts);
        assert_eq!(counts, vec![2, 1, 0, 0, 0]);

        last = dawg.extend('a', last);
        // println!("{:?}", Dot::new(dawg.get_graph()));
        map.clear();
        map.build(&dawg);
        counts = vec![0; 5];
        map.compute_counts(&dawg, &mut counts);
        assert_eq!(counts, vec![3, 2, 1, 0, 0]);
    }

}
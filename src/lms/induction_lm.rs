use dawg::Dawg;
use graph::indexing::NodeIndex;
use lms::LM;
use weight::Weight;

pub struct InductionLM {
    pub name: String,
    train_lm: Box<dyn LM>,
    dawg: Dawg<usize>,
    delta: f64,
    state: NodeIndex,
    last: NodeIndex,
}

impl LM for InductionLM {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    fn reset(&mut self, dawg: &Dawg<usize>) {
        self.dawg = Dawg::new();
        self.state = self.dawg.get_initial();
        self.last = self.dawg.get_initial();
        self.train_lm.reset(dawg);
    }

    fn get_probability(&self, dawg: &Dawg<usize>, label: usize, good_turing: f64) -> f64 {
        self.get_probability_interp(dawg, self.state, label, good_turing)
    }

    fn update(&mut self, dawg: &Dawg<usize>, label: usize) {
        self.last = self.dawg.extend(label, self.last);
        self.state = self.dawg.transition(self.state, label, true).unwrap();
        self.train_lm.update(dawg, label);
        // self.good_turing = good_turing_estimate(&self.dawg, self.dawg.node_count() - 1);
    }
}

impl InductionLM {
    // Don't use smoothing, just interpolate!!!s

    pub fn new(name: String, train_lm: Box<dyn LM>, delta: f64) -> Self {
        let dawg = Dawg::new();
        let state = dawg.get_initial();
        let last = dawg.get_initial();
        Self {
            name,
            train_lm,
            dawg,
            delta,
            state,
            last,
        }
    }

    // Backoff with Kneser-Ney smoothing
    pub fn get_probability_interp(
        &self,
        dawg: &Dawg<usize>,
        state: NodeIndex,
        label: usize,
        good_turing: f64,
    ) -> f64 {
        // if self.kn_max_n >= 0 {
        //     let n: u64 = self.kn_max_n.try_into().unwrap();
        //     let graph = self.dawg.get_graph();
        //     // TODO: Can make this more efficient by computing once and passing.
        //     while n < self.dawg.get_length(state) + 1 {
        //         match graph[state].get_failure() {
        //             Some(next_state) => {
        //                 state = next_state;
        //             },
        //             None => {break},
        //         }
        //     }
        // }

        let count = match self.dawg.transition(state, label, false) {
            Some(next_state) => self.dawg.get_weight(next_state).get_count(),
            None => 0,
        };
        let sum_count = self.dawg.get_weight(state).get_count();

        let back_prob = match self.dawg.get_weight(state).get_failure() {
            Some(fstate) => self.get_probability_interp(dawg, fstate, label, good_turing),
            None => self.train_lm.get_probability(dawg, label, good_turing),
        };

        let graph = self.dawg.get_graph();
        if graph.n_edges(state) == 0 {
            return back_prob;
        }
        (1. - self.delta) * (count as f64) / (sum_count as f64) + self.delta * back_prob
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use dawg::Dawg;
    use tokenize::{TokenIndex, Tokenize};

    use graph::indexing::NodeIndex;
    use graph::vec_graph::dot::Dot;

    use lms::induction_lm::InductionLM;
    use lms::kn_lm::KNLM;
    use lms::LM;

    #[test]
    fn test_get_probability_ab() {
        let tokens = vec!["a", "b"];
        let mut index: TokenIndex<usize> = TokenIndex::new();
        let indices: Vec<_> = tokens.iter().map(|x| index.add(x)).collect();

        let mut dawg = Dawg::new();
        dawg.build(&indices);

        let base_lm = KNLM::new("unigram".to_string(), 0.0, 0, 0);
        let mut lm = InductionLM::new("lm".to_string(), Box::new(base_lm), 0.5);
        let a = index.index("a");
        let b = index.index("b");

        assert_eq!(lm.state.index(), 0);
        // No edges, skip interpolation.
        assert_eq!(lm.get_probability(&dawg, a, 0.), 1. / 3.);
        assert_eq!(lm.get_probability(&dawg, b, 0.), 1. / 3.);
        lm.update(&dawg, a);
        assert_eq!(lm.state.index(), 1);
        // 1/2 * (1/2 + 1/3)
        assert_eq!(lm.get_probability(&dawg, a, 0.), 0.41666666666666663);
        assert_eq!(lm.get_probability(&dawg, b, 0.), 1. / 6.);
        lm.update(&dawg, b);
        // println!("{:?}", Dot::new(lm.dawg.get_graph()));
        assert_eq!(lm.state.index(), 2);
        assert_eq!(lm.get_probability(&dawg, a, 0.), 1. / 3.);
        assert_eq!(lm.get_probability(&dawg, b, 0.), 1. / 3.);
        lm.update(&dawg, a);
        assert_eq!(lm.state.index(), 3);
        // Now b is more likely!
        assert_eq!(lm.get_probability(&dawg, a, 0.), 0.20833333333333331);
        assert_eq!(lm.get_probability(&dawg, b, 0.), 0.3958333333333333);
    }
}

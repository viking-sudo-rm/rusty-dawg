pub mod induction_lm;
pub mod kn_lm;

use crate::weight::weight40::DefaultWeight;
use dawg::Dawg;

pub trait LM {
    fn get_name(&self) -> &str;

    fn reset(&mut self, dawg: &Dawg<usize, DefaultWeight>);

    fn get_probability(
        &self,
        dawg: &Dawg<usize, DefaultWeight>,
        label: usize,
        good_turing: f64,
    ) -> f64;

    fn update(&mut self, dawg: &Dawg<usize, DefaultWeight>, label: usize);
}

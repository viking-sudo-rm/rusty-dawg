pub mod kn_lm;
pub mod induction_lm;

use dawg::Dawg;

pub trait LM {
    fn get_name(&self) -> &str;

    fn reset(&mut self, dawg: &Dawg<usize>);

    fn get_probability(&self, dawg: &Dawg<usize>, label: usize, good_turing: f32) -> f32;

    fn update(&mut self, dawg: &Dawg<usize>, label: usize);
}
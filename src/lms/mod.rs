pub mod induction_lm;
pub mod kn_lm;

use std::fmt::Debug;
use std::io::Write;
use std::marker::Copy;
use serde::Serialize;
use serde::Deserialize;

use crate::weight::weight40::DefaultWeight;
use dawg::Dawg;

pub trait LM {
    fn get_name(&self) -> &str;

    fn reset(&mut self, dawg: &Dawg<u16, DefaultWeight>);

    fn get_probability(
        &self,
        dawg: &Dawg<u16, DefaultWeight>,
        label: u16,
        good_turing: f64,
    ) -> f64;

    fn update(&mut self, dawg: &Dawg<u16, DefaultWeight>, label: u16);
}

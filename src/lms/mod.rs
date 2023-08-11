pub mod induction_lm;
pub mod kn_lm;

use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::io::Write;
use std::marker::Copy;

use crate::weight::weight40::DefaultWeight;
use dawg::Dawg;

pub trait LM<E>
where
    E: Eq + serde::Serialize + Copy + Debug,
{
    fn get_name(&self) -> &str;

    fn reset(&mut self, dawg: &Dawg<E, DefaultWeight>);

    fn get_probability(&self, dawg: &Dawg<E, DefaultWeight>, label: E, good_turing: f64) -> f64;

    fn update(&mut self, dawg: &Dawg<E, DefaultWeight>, label: E);
}

use pyo3::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::cdawg_state::CdawgState;

use rusty_dawg::cdawg;
use rusty_dawg::graph::indexing::DefaultIx;
use rusty_dawg::weight::DefaultWeight;

#[pyclass(unsendable)]
pub struct Cdawg {
    cdawg: cdawg::Cdawg<DefaultWeight, DefaultIx>,
}

// Wrap the normal Dawg class with a Python interface.
#[pymethods]
impl Cdawg {
    #[new]
    pub fn new(tokens: Vec<u16>) -> Self {
        let tokens_rc = Rc::new(RefCell::new(tokens));
        Self {
            cdawg: cdawg::Cdawg::new(tokens_rc),
        }
    }

    pub fn build(&mut self) {
        self.cdawg.build();
    }

    pub fn get_initial(&self) -> CdawgState {
        CdawgState {
            cs: self.cdawg.get_initial(),
        }
    }

    pub fn transition_and_count(&self, cs: CdawgState, token: u16) -> CdawgState {
        CdawgState {
            cs: self.cdawg.transition_and_count(cs.cs, token),
        }
    }
}

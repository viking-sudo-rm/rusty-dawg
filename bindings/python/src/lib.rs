use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub mod dawg;
pub mod disk_dawg;

use dawg::Dawg;
use disk_dawg::DiskDawg;

/// A Python module implemented in Rust.
#[pymodule]
fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Dawg>()?;
    m.add_class::<DiskDawg>()?;
    // Here's how you would add a function:
    // m.add_wrapped(wrap_pyfunction!(good_turing_estimate))?;
    Ok(())
}


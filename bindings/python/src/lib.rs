use pyo3::prelude::*;

pub mod dawg;
// pub mod disk_dawg;  // FIXME(#53): Vector type used by disk-backed DAWG is not thread-safe.

use dawg::Dawg;
// use disk_dawg::DiskDawg;

/// A Python module implemented in Rust.
#[pymodule]
fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Dawg>()?;
    // m.add_class::<DiskDawg>()?;
    Ok(())
}

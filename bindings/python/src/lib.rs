use pyo3::prelude::*;

pub mod cdawg;
pub mod cdawg_state;
pub mod dawg;
pub mod disk_dawg;
pub mod disk_vec;

use cdawg::Cdawg;
use cdawg_state::CdawgState;
use dawg::Dawg;
use disk_dawg::DiskDawg;
use disk_vec::DiskVec;

/// A Python module implemented in Rust.
#[pymodule]
fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Cdawg>()?;
    m.add_class::<CdawgState>()?;
    m.add_class::<Dawg>()?;
    m.add_class::<DiskDawg>()?;
    m.add_class::<DiskVec>()?;
    Ok(())
}

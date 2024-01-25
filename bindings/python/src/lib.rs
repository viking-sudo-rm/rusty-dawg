use pyo3::prelude::*;

pub mod cdawg;
pub mod cdawg_state;
pub mod dawg;
pub mod disk_cdawg;
pub mod disk_dawg;

use cdawg::Cdawg;
use cdawg_state::CdawgState;
use dawg::Dawg;
use disk_cdawg::DiskCdawg;
use disk_dawg::DiskDawg;

/// A Python module implemented in Rust.
#[pymodule]
fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Cdawg>()?;
    m.add_class::<CdawgState>()?;
    m.add_class::<Dawg>()?;
    m.add_class::<DiskCdawg>()?;
    m.add_class::<DiskDawg>()?;
    Ok(())
}

use pyo3::prelude::*;
use pyo3::types::PyType;

use rusty_dawg::graph::memory_backing::disk_backing::disk_vec;

#[pyclass]
pub struct DiskVec {
    vec: disk_vec::DiskVec<u16>,
}

#[pymethods]
impl DiskVec {
    #[classmethod]
    pub fn load(_cls: &PyType, path: String, max_size: usize) -> PyResult<Self> {
        Ok(Self {
            vec: disk_vec::DiskVec::new(path, max_size)?,
        })
    }
}

impl DiskVec {
    pub fn get_disk_vec(&self) -> &disk_vec::DiskVec<u16> {
        &self.vec
    }
}

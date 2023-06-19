// // use pyo3::prelude::*;
// use pyo3::prelude::{Python, PyResult, PyModule};

// #[pyfunction]
// fn multiply(a: isize, b: isize) -> PyResult<isize> {
//     Ok(a * b)
// }

// #[pymodule]
// fn rusty_dawg(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(multiply, m)?)?;
//     Ok(())
// }
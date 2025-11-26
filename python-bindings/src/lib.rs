use pyo3::prelude::*;
use dinja_core::hello;

/// Python bindings for dinja
/// 
/// This module provides a thin wrapper around the core Rust library
/// using PyO3 for Python integration.

/// A Python-callable version of the hello function
/// 
/// # Examples
/// 
/// ```python
/// import dinja
/// 
/// greeting = dinja.hello("World")
/// print(greeting)  # "Hello, World!"
/// ```
#[pyfunction]
fn hello_py(name: &str) -> String {
    hello(name)
}

/// The dinja Python module
#[pymodule]
fn dinja<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_py, m)?)?;
    Ok(())
}


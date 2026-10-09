//! Python binding for PortableMC.

#![deny(unsafe_op_in_unsafe_fn)]

mod uuid;
mod err;
mod handler;

mod msa;

mod installer;
mod base;
mod mojang;
mod fabric;
mod forge;

mod cli;

use pyo3::prelude::*;


#[pymodule]
#[pyo3(name = "_portablemc")]
fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {

    add_submodule(m, "msa", msa::py_module)?;
    add_submodule(m, "base", base::py_module)?;
    add_submodule(m, "mojang", mojang::py_module)?;
    add_submodule(m, "fabric", fabric::py_module)?;
    add_submodule(m, "forge", forge::py_module)?;

    m.add_function(wrap_pyfunction!(cli::py_cli_main, m)?)?;
    
    Ok(())

}

/// Add a submodule to the native module, also registered in `sys.modules` under its
/// full name so that it can be imported from, such as `portablemc._portablemc.base`.
fn add_submodule<'py>(
    m: &Bound<'py, PyModule>,
    name: &str,
    init: fn(&Bound<'py, PyModule>) -> PyResult<()>,
) -> PyResult<()> {

    let py = m.py();
    let full_name = format!("{}.{name}", m.name()?);

    let submodule = PyModule::new(py, &full_name)?;
    init(&submodule)?;
    m.add(name, &submodule)?;

    py.import("sys")?
        .getattr("modules")?
        .set_item(full_name, submodule)?;

    Ok(())

}

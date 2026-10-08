//! Embedded command line interface, used by the Python package's console script.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;
use pyo3::prelude::*;

use portablemc_cli::cmd;
use portablemc_cli::parse::CliArgs;


/// Run the CLI with the given arguments, the first one being the program name, and 
/// return the exit code. Note that the CLI may exit the process by itself, for example
/// when printing help or on Ctrl-C.
#[pyfunction]
#[pyo3(name = "_cli_main")]
pub(super) fn py_cli_main(py: Python<'_>, args: Vec<OsString>) -> i32 {
    let args = CliArgs::parse_from(args);
    let code = py.detach(|| cmd::main(&args));
    if code == ExitCode::SUCCESS { 0 } else { 1 }
}

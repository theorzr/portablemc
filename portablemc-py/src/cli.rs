//! Embedded command line interface, used by the Python package's console script.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::{CommandFactory, FromArgMatches};
use pyo3::{intern, prelude::*};

use portablemc_cli::cmd;
use portablemc_cli::parse::CliArgs;


/// Run the CLI with the given arguments, the first one being the program name, and
/// return the exit code. Note that the CLI may exit the process by itself, for example
/// when printing help or on Ctrl-C.
#[pyfunction]
#[pyo3(name = "_cli_main")]
pub(super) fn py_cli_main(py: Python<'_>, args: Vec<OsString>) -> PyResult<i32> {

    // Extend the long version with the platform, in the same format as the binary
    // distributions, it's leaked because clap requires a static string.
    let mut command = CliArgs::command();
    let long_version = format!("{}\nplatform: {}",
        command.get_long_version().unwrap_or_default(),
        python_platform(py)?);
    command = command.long_version(&*long_version.leak());

    let mut matches = command.get_matches_from(args);
    let args = CliArgs::from_arg_matches_mut(&mut matches)
        .unwrap_or_else(|e| e.format(&mut CliArgs::command()).exit());

    let code = py.detach(|| cmd::main(&args));
    Ok(if code == ExitCode::SUCCESS { 0 } else { 1 })

}

/// Describe the running Python interpreter, for example "Python 3.12.4 (CPython) on linux".
fn python_platform(py: Python<'_>) -> PyResult<String> {

    let mod_platform = PyModule::import(py, intern!(py, "platform"))?;
    let mod_sys = PyModule::import(py, intern!(py, "sys"))?;

    let version = mod_platform.call_method0(intern!(py, "python_version"))?.extract::<String>()?;
    let implementation = mod_platform.call_method0(intern!(py, "python_implementation"))?.extract::<String>()?;
    let os = mod_sys.getattr(intern!(py, "platform"))?.extract::<String>()?;

    Ok(format!("Python {version} ({implementation}) on {os}"))

}

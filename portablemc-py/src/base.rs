use std::path::{Path, PathBuf};
use std::sync::Arc;

use pyo3::exceptions::PyValueError;
use pyo3::types::{IntoPyDict, PyBytes, PyList};
use pyo3::{intern, prelude::*};

use portablemc::base::{default_main_dir, Installer, Game, JvmPolicy, LoadedVersion, VersionChannel, LoadedLibrary, LibraryDownload};
use portablemc::maven::Gav;

use crate::installer::{GenericInstaller, SharedInstaller};
use crate::{err, handler};


/// Define the `_portablemc.base` submodule.
pub(super) fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyJvmPolicy>()?;
    m.add_class::<PyInstaller>()?;
    m.add_class::<PyHandler>()?;
    m.add_class::<PyVersionChannel>()?;
    m.add_class::<PyLoadedVersion>()?;
    m.add_class::<PyLoadedLibrary>()?;
    m.add_class::<PyLibraryDownload>()?;
    m.add_class::<PyGame>()?;
    m.add_function(wrap_pyfunction!(py_default_main_dir, m)?)?;
    err::add_base(m)?;
    Ok(())
}

#[pyfunction]
#[pyo3(name = "default_main_dir")]
fn py_default_main_dir() -> Option<&'static Path> {
    default_main_dir()
}

#[pyclass(name = "JvmPolicy", module = "portablemc.base", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyJvmPolicy {
    System,
    Mojang,
    SystemThenMojang,
    MojangThenSystem,
}

#[derive(FromPyObject, IntoPyObject)]
pub enum PyJvmPolicyUnion {
    Static(PathBuf),
    Policy(PyJvmPolicy),
}

#[pyclass(name = "Installer", module = "portablemc.base", frozen, subclass)]
pub struct PyInstaller(pub Arc<SharedInstaller>);

#[pymethods]
impl PyInstaller {

    #[new]
    fn __new__(version: &str) -> Self {

        let inst = SharedInstaller::new(
            GenericInstaller::Base(Installer::new(version.to_string()))
        );

        Self(inst)

    }

    fn __repr__(&self) -> PyResult<String> {
        let guard = self.0.lock()?;
        Ok(format!("<portablemc.base.Installer version={:?}>", guard.base().version()))
    }

    #[getter]
    fn version(&self) -> PyResult<String> {
        Ok(self.0.lock()?.base().version().to_string())
    }

    #[setter]
    fn set_version(&self, version: String) -> PyResult<()> {
        self.0.lock()?.base_mut().set_version(version);
        Ok(())
    }

    #[getter]
    fn versions_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().versions_dir().to_path_buf())
    }

    #[setter]
    fn set_versions_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_versions_dir(dir);
        Ok(())
    }

    #[getter]
    fn libraries_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().libraries_dir().to_path_buf())
    }

    #[setter]
    fn set_libraries_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_libraries_dir(dir);
        Ok(())
    }

    #[getter]
    fn assets_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().assets_dir().to_path_buf())
    }

    #[setter]
    fn set_assets_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_assets_dir(dir);
        Ok(())
    }

    #[getter]
    fn jvm_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().jvm_dir().to_path_buf())
    }

    #[setter]
    fn set_jvm_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_jvm_dir(dir);
        Ok(())
    }

    #[getter]
    fn bin_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().bin_dir().to_path_buf())
    }

    #[setter]
    fn set_bin_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_bin_dir(dir);
        Ok(())
    }

    #[getter]
    fn mc_dir(&self) -> PyResult<PathBuf> {
        Ok(self.0.lock()?.base().mc_dir().to_path_buf())
    }

    #[setter]
    fn set_mc_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_mc_dir(dir);
        Ok(())
    }

    // No setter because it's a compound function, setting all paths below.
    fn set_main_dir(&self, dir: PathBuf) -> PyResult<()> {
        self.0.lock()?.base_mut().set_main_dir(dir);
        Ok(())
    }

    #[getter]
    fn strict_assets_check(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.base().strict_assets_check())
    }

    #[setter]
    fn set_strict_assets_check(&self, strict: bool) -> PyResult<()> {
        self.0.lock()?.base_mut().set_strict_assets_check(strict);
        Ok(())
    }

    #[getter]
    fn strict_libraries_check(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.base().strict_libraries_check())
    }

    #[setter]
    fn set_strict_libraries_check(&self, strict: bool) -> PyResult<()> {
        self.0.lock()?.base_mut().set_strict_libraries_check(strict);
        Ok(())
    }

    #[getter]
    fn strict_jvm_check(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.base().strict_jvm_check())
    }

    #[setter]
    fn set_strict_jvm_check(&self, strict: bool) -> PyResult<()> {
        self.0.lock()?.base_mut().set_strict_jvm_check(strict);
        Ok(())
    }

    #[getter]
    fn jvm_policy(&self) -> PyResult<PyJvmPolicyUnion> {
        Ok(match self.0.lock()?.base().jvm_policy() {
            JvmPolicy::Static(file) => PyJvmPolicyUnion::Static(file.clone()),
            JvmPolicy::System => PyJvmPolicyUnion::Policy(PyJvmPolicy::System),
            JvmPolicy::Mojang => PyJvmPolicyUnion::Policy(PyJvmPolicy::Mojang),
            JvmPolicy::SystemThenMojang => PyJvmPolicyUnion::Policy(PyJvmPolicy::SystemThenMojang),
            JvmPolicy::MojangThenSystem => PyJvmPolicyUnion::Policy(PyJvmPolicy::MojangThenSystem),
        })
    }

    #[setter]
    fn set_jvm_policy(&self, policy: PyJvmPolicyUnion) -> PyResult<()> {
        self.0.lock()?.base_mut().set_jvm_policy(match policy {
            PyJvmPolicyUnion::Static(file) => JvmPolicy::Static(file),
            PyJvmPolicyUnion::Policy(PyJvmPolicy::System) => JvmPolicy::System,
            PyJvmPolicyUnion::Policy(PyJvmPolicy::Mojang) => JvmPolicy::Mojang,
            PyJvmPolicyUnion::Policy(PyJvmPolicy::SystemThenMojang) => JvmPolicy::SystemThenMojang,
            PyJvmPolicyUnion::Policy(PyJvmPolicy::MojangThenSystem) => JvmPolicy::MojangThenSystem,
        });
        Ok(())
    }

    #[getter]
    fn launcher_name(&self) -> PyResult<String> {
        Ok(self.0.lock()?.base().launcher_name().to_string())
    }

    #[setter]
    fn set_launcher_name(&self, name: String) -> PyResult<()> {
        self.0.lock()?.base_mut().set_launcher_name(name);
        Ok(())
    }

    #[getter]
    fn launcher_version(&self) -> PyResult<String> {
        Ok(self.0.lock()?.base().launcher_version().to_string())
    }

    #[setter]
    fn set_launcher_version(&self, version: String) -> PyResult<()> {
        self.0.lock()?.base_mut().set_launcher_version(version);
        Ok(())
    }

    #[pyo3(signature = (handler = None))]
    fn install(&self, py: Python<'_>, handler: Option<Py<PyAny>>) -> PyResult<PyGame> {
        let mut guard = self.0.lock_install()?;
        let inst = guard.base_mut();
        handler::install(py, handler, |h| inst.install(h), err::from_base)
    }

}

/// Handler for base installer events, every method does nothing by default and can be
/// overridden by subclasses.
#[pyclass(name = "Handler", module = "portablemc.base", frozen, subclass)]
pub struct PyHandler;

#[pymethods]
#[allow(unused_variables)]
impl PyHandler {

    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn __new__(args: &Bound<'_, PyAny>, kwargs: Option<&Bound<'_, PyAny>>) -> Self {
        Self
    }

    fn filter_features(&self, features: &Bound<'_, PyAny>) {}
    fn loaded_features(&self, features: &Bound<'_, PyAny>) {}
    fn load_hierarchy(&self, root_version: &Bound<'_, PyAny>) {}
    fn loaded_hierarchy(&self, hierarchy: &Bound<'_, PyAny>) {}
    fn load_version(&self, version: &Bound<'_, PyAny>, file: &Bound<'_, PyAny>) {}
    fn need_version(&self, version: &Bound<'_, PyAny>, file: &Bound<'_, PyAny>) -> bool { false }
    fn loaded_version(&self, version: &Bound<'_, PyAny>, file: &Bound<'_, PyAny>) {}
    fn load_client(&self) {}
    fn loaded_client(&self, file: &Bound<'_, PyAny>) {}
    fn load_libraries(&self) {}
    fn filter_libraries(&self, libraries: &Bound<'_, PyAny>) {}
    fn loaded_libraries(&self, libraries: &Bound<'_, PyAny>) {}
    fn filter_libraries_files(&self, class_files: &Bound<'_, PyAny>, natives_files: &Bound<'_, PyAny>) {}
    fn loaded_libraries_files(&self, class_files: &Bound<'_, PyAny>, natives_files: &Bound<'_, PyAny>) {}
    fn no_logger(&self) {}
    fn load_logger(&self, id: &Bound<'_, PyAny>) {}
    fn loaded_logger(&self, id: &Bound<'_, PyAny>) {}
    fn no_assets(&self) {}
    fn load_assets(&self, id: &Bound<'_, PyAny>) {}
    fn loaded_assets(&self, id: &Bound<'_, PyAny>, count: &Bound<'_, PyAny>) {}
    fn verified_assets(&self, id: &Bound<'_, PyAny>, count: &Bound<'_, PyAny>) {}
    fn load_jvm(&self, major_version: &Bound<'_, PyAny>) {}
    fn found_jvm_system_version(&self, file: &Bound<'_, PyAny>, version: &Bound<'_, PyAny>, compatible: &Bound<'_, PyAny>) {}
    fn warn_jvm_unsupported_dynamic_crt(&self) {}
    fn warn_jvm_unsupported_platform(&self) {}
    fn warn_jvm_missing_distribution(&self) {}
    fn loaded_jvm(&self, file: &Bound<'_, PyAny>, version: &Bound<'_, PyAny>, compatible: &Bound<'_, PyAny>) {}
    fn download_resources(&self) -> bool { false }
    fn downloaded_resources(&self) {}
    fn download_progress(&self, count: &Bound<'_, PyAny>, total_count: &Bound<'_, PyAny>, size: &Bound<'_, PyAny>, total_size: &Bound<'_, PyAny>) {}
    fn extracted_binaries(&self, dir: &Bound<'_, PyAny>) {}

}

#[pyclass(name = "VersionChannel", module = "portablemc.base", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyVersionChannel {
    Release,
    Snapshot,
    Beta,
    Alpha,
}

impl From<VersionChannel> for PyVersionChannel {
    fn from(value: VersionChannel) -> Self {
        match value {
            VersionChannel::Release => Self::Release,
            VersionChannel::Snapshot => Self::Snapshot,
            VersionChannel::Beta => Self::Beta,
            VersionChannel::Alpha => Self::Alpha,
        }
    }
}

#[pyclass(name = "LoadedVersion", module = "portablemc.base", frozen)]
pub struct PyLoadedVersion(pub LoadedVersion);

#[pymethods]
impl PyLoadedVersion {

    fn __repr__(&self) -> String {
        format!("<portablemc.base.LoadedVersion name={:?} dir={:?}>", self.0.name(), self.0.dir())
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.name()
    }

    #[getter]
    fn dir(&self) -> &Path {
        self.0.dir()
    }

    #[getter]
    fn channel(&self) -> Option<PyVersionChannel> {
        self.0.channel().map(PyVersionChannel::from)
    }

}

#[pyclass(name = "LoadedLibrary", module = "portablemc.base")]
pub struct PyLoadedLibrary(pub LoadedLibrary);

#[pymethods]
impl PyLoadedLibrary {

    #[new]
    #[pyo3(signature = (name, path = None, download = None, natives = false))]
    fn __new__(name: &str, path: Option<PathBuf>, download: Option<PyRef<'_, PyLibraryDownload>>, natives: bool) -> PyResult<Self> {
        Ok(Self(LoadedLibrary {
            name: parse_gav(name)?,
            path,
            download: download.map(|d| d.0.clone()),
            natives,
        }))
    }

    fn __repr__(&self) -> String {
        format!("<portablemc.base.LoadedLibrary name={:?} path={} natives={}>",
            self.0.name.as_str(),
            self.0.path.as_ref().map(|p| format!("{p:?}")).as_deref().unwrap_or("None"),
            if self.0.natives { "True" } else { "False" })
    }

    #[getter]
    fn name(&self) -> &str {
        self.0.name.as_str()
    }

    #[setter]
    fn set_name(&mut self, name: &str) -> PyResult<()> {
        self.0.name = parse_gav(name)?;
        Ok(())
    }

    #[getter]
    fn path(&self) -> Option<&Path> {
        self.0.path.as_deref()
    }

    #[setter]
    fn set_path(&mut self, path: Option<PathBuf>) {
        self.0.path = path;
    }

    #[getter]
    fn download(&self) -> Option<PyLibraryDownload> {
        self.0.download.clone().map(PyLibraryDownload)
    }

    #[setter]
    fn set_download(&mut self, download: Option<PyRef<'_, PyLibraryDownload>>) {
        self.0.download = download.map(|d| d.0.clone());
    }

    #[getter]
    fn natives(&self) -> bool {
        self.0.natives
    }

    #[setter]
    fn set_natives(&mut self, natives: bool) {
        self.0.natives = natives;
    }

}

fn parse_gav(name: &str) -> PyResult<Gav> {
    name.parse::<Gav>()
        .map_err(|()| PyValueError::new_err(format!("invalid library name: {name:?}")))
}

#[pyclass(name = "LibraryDownload", module = "portablemc.base", frozen)]
pub struct PyLibraryDownload(pub LibraryDownload);

#[pymethods]
impl PyLibraryDownload {

    #[new]
    #[pyo3(signature = (url, size = None, sha1 = None))]
    fn __new__(url: String, size: Option<u32>, sha1: Option<&[u8]>) -> PyResult<Self> {
        let sha1 = sha1
            .map(|sha1| <[u8; 20]>::try_from(sha1)
                .map_err(|_| PyValueError::new_err("sha1 must be 20 bytes long")))
            .transpose()?;
        Ok(Self(LibraryDownload { url, size, sha1 }))
    }

    fn __repr__(&self) -> String {
        format!("<portablemc.base.LibraryDownload url={:?}>", self.0.url)
    }

    #[getter]
    fn url(&self) -> &str {
        &self.0.url
    }

    #[getter]
    fn size(&self) -> Option<u32> {
        self.0.size
    }

    #[getter]
    fn sha1<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.0.sha1.as_ref().map(|sha1| PyBytes::new(py, sha1))
    }

}

/// An installed game ready to be launched, its arguments lists can be modified in place.
#[pyclass(name = "Game", module = "portablemc.base")]
pub struct PyGame {
    #[pyo3(get, set)]
    jvm_file: PathBuf,
    #[pyo3(get, set)]
    mc_dir: PathBuf,
    #[pyo3(get, set)]
    main_class: String,
    #[pyo3(get)]
    jvm_args: Py<PyList>,
    #[pyo3(get)]
    game_args: Py<PyList>,
}

impl PyGame {

    pub fn new(py: Python<'_>, game: Game) -> PyResult<Self> {
        Ok(Self {
            jvm_file: game.jvm_file,
            mc_dir: game.mc_dir,
            main_class: game.main_class,
            jvm_args: PyList::new(py, game.jvm_args)?.unbind(),
            game_args: PyList::new(py, game.game_args)?.unbind(),
        })
    }

}

#[pymethods]
impl PyGame {

    fn __repr__(&self) -> String {
        format!("<portablemc.base.Game jvm_file={:?} main_class={:?}>", self.jvm_file, self.main_class)
    }

    #[setter]
    fn set_jvm_args(&mut self, py: Python<'_>, args: Vec<String>) -> PyResult<()> {
        self.jvm_args = PyList::new(py, args)?.unbind();
        Ok(())
    }

    #[setter]
    fn set_game_args(&mut self, py: Python<'_>, args: Vec<String>) -> PyResult<()> {
        self.game_args = PyList::new(py, args)?.unbind();
        Ok(())
    }

    /// Return the full command line, starting with the JVM executable.
    fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        let args = PyList::empty(py);
        args.append(&self.jvm_file)?;
        for arg in self.jvm_args.bind(py) {
            args.append(arg)?;
        }
        args.append(&self.main_class)?;
        for arg in self.game_args.bind(py) {
            args.append(arg)?;
        }
        Ok(args)
    }

    /// Return a `subprocess.Popen` partial function with the command line and working
    /// directory already set, further arguments are given to `Popen`.
    fn command<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {

        let mod_subprocess = PyModule::import(py, intern!(py, "subprocess"))?;
        let ty_popen = mod_subprocess.getattr(intern!(py, "Popen"))?;

        let mod_functools = PyModule::import(py, intern!(py, "functools"))?;
        let func_partial = mod_functools.getattr(intern!(py, "partial"))?;

        let kwargs = [("cwd", &self.mc_dir)].into_py_dict(py)?;
        func_partial.call((&ty_popen, self.args(py)?), Some(&kwargs))

    }

}

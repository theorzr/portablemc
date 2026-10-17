use std::sync::{Arc, Mutex};

use pyo3::types::{PyIterator, PyList};
use pyo3::prelude::*;

use portablemc::forge::{InstallReason, Installer, Loader, Repo, RepoVersion, Version};

use crate::installer::{GenericInstaller, SharedInstaller};
use crate::base::PyGame;
use crate::{err, handler};


/// Define the `_portablemc.forge` submodule.
pub(super) fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLoader>()?;
    m.add_class::<PyVersion>()?;
    m.add_class::<PyInstaller>()?;
    m.add_class::<PyHandler>()?;
    m.add_class::<PyInstallReason>()?;
    m.add_class::<PyRepo>()?;
    m.add_class::<PyRepoVersion>()?;
    err::add_forge(m)?;
    Ok(())
}

#[pyclass(name = "Loader", module = "portablemc.forge", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyLoader {
    Forge,
    NeoForge,
}

impl From<PyLoader> for Loader {
    fn from(value: PyLoader) -> Self {
        match value {
            PyLoader::Forge => Loader::Forge,
            PyLoader::NeoForge => Loader::NeoForge,
        }
    }
}

#[pyclass(name = "Version", module = "portablemc.forge", eq, from_py_object)]
#[derive(Clone, PartialEq, Eq)]
enum PyVersion {
    Stable(String),
    Unstable(String),
    Name(String),
}

impl From<PyVersion> for Version {
    fn from(value: PyVersion) -> Self {
        match value {
            PyVersion::Stable(game_version) => Version::Stable(game_version),
            PyVersion::Unstable(game_version) => Version::Unstable(game_version),
            PyVersion::Name(name) => Version::Name(name),
        }
    }
}

#[pyclass(name = "Installer", module = "portablemc.forge", frozen, subclass, extends = crate::mojang::PyInstaller)]
pub(crate) struct PyInstaller(pub(crate) Arc<SharedInstaller>);

#[pymethods]
impl PyInstaller {

    #[new]
    fn __new__(loader: PyLoader, version: PyVersion) -> PyClassInitializer<Self> {

        let inst = SharedInstaller::new(
            GenericInstaller::Forge(Installer::new(loader.into(), version))
        );
        
        PyClassInitializer::from(crate::base::PyInstaller(Arc::clone(&inst)))
            .add_subclass(crate::mojang::PyInstaller(Arc::clone(&inst)))
            .add_subclass(Self(inst))

    }

    fn __repr__(&self) -> PyResult<String> {
        let guard = self.0.lock()?;
        let inst = guard.forge();
        Ok(format!("<portablemc.forge.Installer loader=Loader.{:?} version=Version.{:?}>", inst.loader(), inst.version()))
    }

    #[getter]
    fn loader(&self) -> PyResult<PyLoader> {
        Ok(match self.0.lock()?.forge().loader() {
            Loader::Forge => PyLoader::Forge,
            Loader::NeoForge => PyLoader::NeoForge,
        })
    }

    #[setter]
    fn set_loader(&self, loader: PyLoader) -> PyResult<()> {
        self.0.lock()?.forge_mut().set_loader(loader.into());
        Ok(())
    }

    #[getter]
    fn version(&self) -> PyResult<PyVersion> {
        Ok(match self.0.lock()?.forge().version() {
            Version::Stable(game_version) => PyVersion::Stable(game_version.clone()),
            Version::Unstable(game_version) => PyVersion::Unstable(game_version.clone()),
            Version::Name(name) => PyVersion::Name(name.clone()),
        })
    }

    #[setter]
    fn set_version(&self, version: PyVersion) -> PyResult<()> {
        self.0.lock()?.forge_mut().set_version(version);
        Ok(())
    }

    #[pyo3(signature = (handler = None))]
    fn install(&self, py: Python<'_>, handler: Option<Py<PyAny>>) -> PyResult<PyGame> {
        let mut guard = self.0.lock_install()?;
        let inst = guard.forge_mut();
        handler::install(py, handler, |h| inst.install(h), err::from_forge)
    }

}

/// Handler for Forge installer events, every method does nothing by default and can 
/// be overridden by subclasses.
#[pyclass(name = "Handler", module = "portablemc.forge", frozen, subclass, extends = crate::mojang::PyHandler)]
pub struct PyHandler;

#[pymethods]
#[allow(unused_variables)]
impl PyHandler {

    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn __new__(args: &Bound<'_, PyAny>, kwargs: Option<&Bound<'_, PyAny>>) -> PyClassInitializer<Self> {
        PyClassInitializer::from(crate::base::PyHandler)
            .add_subclass(crate::mojang::PyHandler)
            .add_subclass(Self)
    }

    fn installing(&self, tmp_dir: &Bound<'_, PyAny>, reason: &Bound<'_, PyAny>) {}
    fn fetch_installer(&self, version: &Bound<'_, PyAny>) {}
    fn fetched_installer(&self, version: &Bound<'_, PyAny>) {}
    fn installing_game(&self) {}
    fn fetch_installer_libraries(&self) {}
    fn fetched_installer_libraries(&self) {}
    fn run_installer_processor(&self, name: &Bound<'_, PyAny>, task: &Bound<'_, PyAny>) {}
    fn installed(&self) {}

}

#[pyclass(name = "InstallReason", module = "portablemc.forge", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyInstallReason {
    MissingVersionMetadata,
    MissingCoreLibrary,
    MissingClientExtra,
    MissingClientSrg,
    MissingPatchedClient,
    MissingUniversalClient,
}

impl From<InstallReason> for PyInstallReason {
    fn from(value: InstallReason) -> Self {
        match value {
            InstallReason::MissingVersionMetadata => Self::MissingVersionMetadata,
            InstallReason::MissingCoreLibrary => Self::MissingCoreLibrary,
            InstallReason::MissingClientExtra => Self::MissingClientExtra,
            InstallReason::MissingClientSrg => Self::MissingClientSrg,
            InstallReason::MissingPatchedClient => Self::MissingPatchedClient,
            InstallReason::MissingUniversalClient => Self::MissingUniversalClient,
        }
    }
}

/// The versions repository of Forge or NeoForge.
#[pyclass(name = "Repo", module = "portablemc.forge", frozen)]
struct PyRepo {
    loader: PyLoader,
    inner: Mutex<Repo>,
}

#[pymethods]
impl PyRepo {

    #[staticmethod]
    fn request(py: Python<'_>, loader: PyLoader) -> PyResult<Self> {
        py.detach(|| Repo::request(loader.into()))
            .map(|repo| Self { loader, inner: Mutex::new(repo) })
            .map_err(|e| err::from_forge(py, e))
    }

    fn __repr__(&self) -> String {
        format!("<portablemc.forge.Repo loader=Loader.{:?}>", Loader::from(self.loader))
    }

    #[getter]
    fn loader(&self) -> PyLoader {
        self.loader
    }

    /// Iterate over all loader versions, the order is not consistent between Forge and
    /// NeoForge.
    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let repo = self.inner.lock().unwrap();
        PyList::new(py, repo.iter().map(PyRepoVersion::from))?.try_iter()
    }

    fn find_by_name(&self, name: &str) -> Option<PyRepoVersion> {
        self.inner.lock().unwrap().find_by_name(name).map(PyRepoVersion::from)
    }

    #[pyo3(signature = (game_version, stable = true))]
    fn find_latest(&self, game_version: &str, stable: bool) -> Option<PyRepoVersion> {
        self.inner.lock().unwrap().find_latest(game_version, stable).map(PyRepoVersion::from)
    }

}

/// A loader version in a Forge or NeoForge repository.
#[pyclass(name = "RepoVersion", module = "portablemc.forge", frozen, get_all)]
struct PyRepoVersion {
    name: String,
    game_version: String,
    stable: bool,
}

impl From<RepoVersion<'_>> for PyRepoVersion {
    fn from(value: RepoVersion<'_>) -> Self {
        Self {
            name: value.name().to_string(),
            game_version: value.game_version().to_string(),
            stable: value.is_stable(),
        }
    }
}

#[pymethods]
impl PyRepoVersion {
    fn __repr__(&self) -> String {
        format!("<portablemc.forge.RepoVersion name={:?} game_version={:?} stable={}>", 
            self.name, self.game_version, if self.stable { "True" } else { "False" })
    }
}

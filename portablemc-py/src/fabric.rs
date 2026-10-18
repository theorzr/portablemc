use std::sync::Arc;
use std::fmt::Write as _;

use pyo3::prelude::*;

use portablemc::fabric::{Api, GameVersion, Installer, Loader, LoaderVersion};

use crate::installer::{GenericInstaller, SharedInstaller};
use crate::base::PyGame;
use crate::{err, handler};


/// Define the `_portablemc.fabric` submodule.
pub(super) fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyLoader>()?;
    m.add_class::<PyGameVersion>()?;
    m.add_class::<PyLoaderVersion>()?;
    m.add_class::<PyInstaller>()?;
    m.add_class::<PyHandler>()?;
    m.add_class::<PyApi>()?;
    m.add_class::<PyApiGameVersion>()?;
    m.add_class::<PyApiLoaderVersion>()?;
    err::add_fabric(m)?;
    Ok(())
}

#[pyclass(name = "Loader", module = "portablemc.fabric", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyLoader {
    Fabric,
    Quilt,
    LegacyFabric,
    Babric,
}

impl From<PyLoader> for Loader {
    fn from(value: PyLoader) -> Self {
        match value {
            PyLoader::Fabric => Loader::Fabric,
            PyLoader::Quilt => Loader::Quilt,
            PyLoader::LegacyFabric => Loader::LegacyFabric,
            PyLoader::Babric => Loader::Babric,
        }
    }
}

#[pyclass(name = "GameVersion", module = "portablemc.fabric", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyGameVersion {
    Stable,
    Unstable,
}

#[derive(FromPyObject, IntoPyObject)]
enum PyGameVersionUnion {
    Version(PyGameVersion),
    Name(String),
}

impl From<PyGameVersionUnion> for GameVersion {
    fn from(value: PyGameVersionUnion) -> Self {
        match value {
            PyGameVersionUnion::Version(PyGameVersion::Stable) => GameVersion::Stable,
            PyGameVersionUnion::Version(PyGameVersion::Unstable) => GameVersion::Unstable,
            PyGameVersionUnion::Name(name) => GameVersion::Name(name),
        }
    }
}

#[pyclass(name = "LoaderVersion", module = "portablemc.fabric", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyLoaderVersion {
    Stable,
    Unstable,
}

#[derive(FromPyObject, IntoPyObject)]
enum PyLoaderVersionUnion {
    Version(PyLoaderVersion),
    Name(String),
}

impl From<PyLoaderVersionUnion> for LoaderVersion {
    fn from(value: PyLoaderVersionUnion) -> Self {
        match value {
            PyLoaderVersionUnion::Version(PyLoaderVersion::Stable) => LoaderVersion::Stable,
            PyLoaderVersionUnion::Version(PyLoaderVersion::Unstable) => LoaderVersion::Unstable,
            PyLoaderVersionUnion::Name(name) => LoaderVersion::Name(name),
        }
    }
}

#[pyclass(name = "Installer", module = "portablemc.fabric", frozen, subclass, extends = crate::mojang::PyInstaller)]
pub(crate) struct PyInstaller(pub(crate) Arc<SharedInstaller>);

#[pymethods]
impl PyInstaller {

    #[new]
    #[pyo3(signature = (loader, game_version = PyGameVersionUnion::Version(PyGameVersion::Stable), loader_version = PyLoaderVersionUnion::Version(PyLoaderVersion::Stable)))]
    fn __new__(loader: PyLoader, game_version: PyGameVersionUnion, loader_version: PyLoaderVersionUnion) -> PyClassInitializer<Self> {

        let inst = SharedInstaller::new(
            GenericInstaller::Fabric(Installer::new(loader.into(), game_version, loader_version))
        );
        
        PyClassInitializer::from(crate::base::PyInstaller(Arc::clone(&inst)))
            .add_subclass(crate::mojang::PyInstaller(Arc::clone(&inst)))
            .add_subclass(Self(inst))

    }

    fn __repr__(&self) -> PyResult<String> {
        
        let guard = self.0.lock()?;
        let inst = guard.fabric();
        let mut buf = format!("<portablemc.fabric.Installer loader=Loader.{:?}", inst.loader());
        
        match inst.game_version() {
            GameVersion::Stable => write!(buf, " game_version=GameVersion.Stable").unwrap(),
            GameVersion::Unstable => write!(buf, " game_version=GameVersion.Unstable").unwrap(),
            GameVersion::Name(name) => write!(buf, " game_version={name:?}").unwrap(),
        }
        
        match inst.loader_version() {
            LoaderVersion::Stable => write!(buf, " loader_version=LoaderVersion.Stable").unwrap(),
            LoaderVersion::Unstable => write!(buf, " loader_version=LoaderVersion.Unstable").unwrap(),
            LoaderVersion::Name(name) => write!(buf, " loader_version={name:?}").unwrap(),
        }

        write!(buf, ">").unwrap();
        Ok(buf)
    }

    #[getter]
    fn loader(&self) -> PyResult<PyLoader> {
        Ok(match self.0.lock()?.fabric().loader() {
            Loader::Fabric => PyLoader::Fabric,
            Loader::Quilt => PyLoader::Quilt,
            Loader::LegacyFabric => PyLoader::LegacyFabric,
            Loader::Babric => PyLoader::Babric,
        })
    }

    #[setter]
    fn set_loader(&self, loader: PyLoader) -> PyResult<()> {
        self.0.lock()?.fabric_mut().set_loader(loader.into());
        Ok(())
    }

    #[getter]
    fn game_version(&self) -> PyResult<PyGameVersionUnion> {
        Ok(match self.0.lock()?.fabric().game_version() {
            GameVersion::Stable => PyGameVersionUnion::Version(PyGameVersion::Stable),
            GameVersion::Unstable => PyGameVersionUnion::Version(PyGameVersion::Unstable),
            GameVersion::Name(name) => PyGameVersionUnion::Name(name.clone()),
        })
    }

    #[setter]
    fn set_game_version(&self, game_version: PyGameVersionUnion) -> PyResult<()> {
        self.0.lock()?.fabric_mut().set_game_version(game_version);
        Ok(())
    }

    #[getter]
    fn loader_version(&self) -> PyResult<PyLoaderVersionUnion> {
        Ok(match self.0.lock()?.fabric().loader_version() {
            LoaderVersion::Stable => PyLoaderVersionUnion::Version(PyLoaderVersion::Stable),
            LoaderVersion::Unstable => PyLoaderVersionUnion::Version(PyLoaderVersion::Unstable),
            LoaderVersion::Name(name) => PyLoaderVersionUnion::Name(name.clone()),
        })
    }

    #[setter]
    fn set_loader_version(&self, loader_version: PyLoaderVersionUnion) -> PyResult<()> {
        self.0.lock()?.fabric_mut().set_loader_version(loader_version);
        Ok(())
    }

    #[pyo3(signature = (handler = None))]
    fn install(&self, py: Python<'_>, handler: Option<Py<PyAny>>) -> PyResult<PyGame> {
        let mut guard = self.0.lock_install()?;
        let inst = guard.fabric_mut();
        handler::install(py, handler, |h| inst.install(h), err::from_fabric)
    }

}

/// Handler for Fabric installer events, every method does nothing by default and can 
/// be overridden by subclasses.
#[pyclass(name = "Handler", module = "portablemc.fabric", frozen, subclass, extends = crate::mojang::PyHandler)]
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

    fn fetch_loader_version(&self, game_version: &Bound<'_, PyAny>, loader_version: &Bound<'_, PyAny>) {}
    fn fetched_loader_version(&self, game_version: &Bound<'_, PyAny>, loader_version: &Bound<'_, PyAny>) {}

}

/// A Fabric-compatible API, used to list the game and loader versions it supports.
#[pyclass(name = "Api", module = "portablemc.fabric", frozen)]
struct PyApi {
    loader: PyLoader,
    inner: Api,
}

#[pymethods]
impl PyApi {

    #[new]
    fn __new__(loader: PyLoader) -> Self {
        Self { loader, inner: Api::new(loader.into()) }
    }

    fn __repr__(&self) -> String {
        format!("<portablemc.fabric.Api loader=Loader.{:?}>", Loader::from(self.loader))
    }

    #[getter]
    fn loader(&self) -> PyLoader {
        self.loader
    }

    fn request_game_versions(&self, py: Python<'_>) -> PyResult<Vec<PyApiGameVersion>> {
        py.detach(|| {
            self.inner.request_game_versions().map(|versions| versions.iter()
                .map(|v| PyApiGameVersion { name: v.name().to_string(), stable: v.is_stable() })
                .collect::<Vec<_>>())
        }).map_err(|e| err::from_fabric(py, e))
    }

    #[pyo3(signature = (game_version = None))]
    fn request_loader_versions(&self, py: Python<'_>, game_version: Option<&str>) -> PyResult<Vec<PyApiLoaderVersion>> {
        py.detach(|| {
            self.inner.request_loader_versions(game_version).map(|versions| versions.iter()
                .map(|v| PyApiLoaderVersion { name: v.name().to_string(), stable: v.is_stable() })
                .collect::<Vec<_>>())
        }).map_err(|e| err::from_fabric(py, e))
    }

}

/// A game version supported by a Fabric-compatible API.
#[pyclass(name = "ApiGameVersion", module = "portablemc.fabric", frozen, get_all)]
struct PyApiGameVersion {
    name: String,
    stable: bool,
}

#[pymethods]
impl PyApiGameVersion {
    fn __repr__(&self) -> String {
        format!("<portablemc.fabric.ApiGameVersion name={:?} stable={}>", self.name, if self.stable { "True" } else { "False" })
    }
}

/// A loader version supported by a Fabric-compatible API.
#[pyclass(name = "ApiLoaderVersion", module = "portablemc.fabric", frozen, get_all)]
struct PyApiLoaderVersion {
    name: String,
    stable: bool,
}

#[pymethods]
impl PyApiLoaderVersion {
    fn __repr__(&self) -> String {
        format!("<portablemc.fabric.ApiLoaderVersion name={:?} stable={}>", self.name, if self.stable { "True" } else { "False" })
    }
}

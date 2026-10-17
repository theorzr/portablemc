use std::sync::Arc;
use std::fmt::Write as _;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use pyo3::exceptions::{PyIndexError, PyValueError};
use pyo3::types::{PyBytes, PyIterator, PyList};
use pyo3::prelude::*;
use regex::Regex;

use portablemc::moj::{FetchExclude, Installer, Manifest, ManifestVersion, QuickPlay, Version};

use crate::base::{PyGame, PyVersionChannel};
use crate::installer::{GenericInstaller, SharedInstaller};
use crate::handler::HandlerAdapter;
use crate::uuid::PyUuid;
use crate::{err, handler, msa};


/// Define the `_portablemc.mojang` submodule.
pub(super) fn py_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVersion>()?;
    m.add_class::<PyQuickPlay>()?;
    m.add_class::<PyFetchExclude>()?;
    m.add_class::<PyInstaller>()?;
    m.add_class::<PyHandler>()?;
    m.add_class::<PyManifest>()?;
    m.add_class::<PyManifestVersion>()?;
    err::add_mojang(m)?;
    Ok(())
}

#[pyclass(name = "Version", module = "portablemc.mojang", eq, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyVersion {
    Release,
    Snapshot,
}

#[derive(FromPyObject, IntoPyObject)]
pub enum PyVersionUnion {
    Version(PyVersion),
    Name(String),
}

impl From<PyVersionUnion> for Version {
    fn from(value: PyVersionUnion) -> Self {
        match value {
            PyVersionUnion::Version(PyVersion::Release) => Version::Release,
            PyVersionUnion::Version(PyVersion::Snapshot) => Version::Snapshot,
            PyVersionUnion::Name(name) => Version::Name(name),
        }
    }
}

#[pyclass(name = "QuickPlay", module = "portablemc.mojang", eq, from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub enum PyQuickPlay {
    Path {
        path: PathBuf,
    },
    Singleplayer {
        name: String,
    },
    Multiplayer {
        host: String,
        port: u16,
    },
    Realms {
        id: String,
    },
}

#[pymethods]
impl PyQuickPlay {
    fn __repr__(&self) -> String {
        match self {
            PyQuickPlay::Path { path } => format!("QuickPlay.Path({path:?})"),
            PyQuickPlay::Singleplayer { name } => format!("QuickPlay.Singleplayer({name:?})"),
            PyQuickPlay::Multiplayer { host, port } => format!("QuickPlay.Multiplayer({host:?}, {port})"),
            PyQuickPlay::Realms { id } => format!("QuickPlay.Realms({id:?})"),
        }
    }
}

#[pyclass(name = "FetchExclude", module = "portablemc.mojang", eq, from_py_object)]
#[derive(Clone, PartialEq, Eq)]
pub enum PyFetchExclude {
    All {},
    Exact {
        name: String,
    },
    Regex {
        pattern: String,
    },
}

#[pymethods]
impl PyFetchExclude {
    fn __repr__(&self) -> String {
        match self {
            PyFetchExclude::All {} => format!("FetchExclude.All()"),
            PyFetchExclude::Exact { name } => format!("FetchExclude.Exact({name:?})"),
            PyFetchExclude::Regex { pattern } => format!("FetchExclude.Regex({pattern:?})"),
        }
    }
}

impl PyFetchExclude {

    fn to_rust(&self) -> PyResult<FetchExclude> {
        Ok(match self {
            PyFetchExclude::All {} => FetchExclude::All,
            PyFetchExclude::Exact { name } => FetchExclude::Exact(name.clone()),
            PyFetchExclude::Regex { pattern } => FetchExclude::Regex(Regex::new(pattern)
                .map_err(|e| PyValueError::new_err(format!("invalid regex: {e}")))?),
        })
    }

}

impl From<&FetchExclude> for PyFetchExclude {
    fn from(value: &FetchExclude) -> Self {
        match value {
            FetchExclude::All => PyFetchExclude::All {},
            FetchExclude::Exact(name) => PyFetchExclude::Exact { name: name.clone() },
            FetchExclude::Regex(regex) => PyFetchExclude::Regex { pattern: regex.as_str().to_string() },
        }
    }
}

#[pyclass(name = "Installer", module = "portablemc.mojang", frozen, subclass, extends = crate::base::PyInstaller)]
pub struct PyInstaller(pub Arc<SharedInstaller>);

#[pymethods]
impl PyInstaller {

    #[new]
    #[pyo3(signature = (version = PyVersionUnion::Version(PyVersion::Release)))]
    fn __new__(version: PyVersionUnion) -> PyClassInitializer<Self> {

        let inst = SharedInstaller::new(
            GenericInstaller::Mojang(Installer::new(version))
        );
        
        PyClassInitializer::from(crate::base::PyInstaller(Arc::clone(&inst)))
            .add_subclass(Self(inst))

    }

    fn __repr__(&self) -> PyResult<String> {

        let guard = self.0.lock()?;
        let inst = guard.mojang();
        let mut buf = format!("<portablemc.mojang.Installer");
        
        match inst.version() {
            Version::Release => write!(buf, " version=Version.Release").unwrap(),
            Version::Snapshot => write!(buf, " version=Version.Snapshot").unwrap(),
            Version::Name(name) => write!(buf, " version={name:?}").unwrap(),
        }

        write!(buf, ">").unwrap();
        Ok(buf)
    }

    #[getter]
    fn version(&self) -> PyResult<PyVersionUnion> {
        Ok(match self.0.lock()?.mojang().version() {
            Version::Release => PyVersionUnion::Version(PyVersion::Release),
            Version::Snapshot => PyVersionUnion::Version(PyVersion::Snapshot),
            Version::Name(name) => PyVersionUnion::Name(name.clone()),
        })
    }

    #[setter]
    fn set_version(&self, version: PyVersionUnion) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_version(version);
        Ok(())
    }

    #[getter]
    fn fetch_excludes(&self) -> PyResult<Vec<PyFetchExclude>> {
        Ok(self.0.lock()?.mojang().fetch_excludes().iter()
            .map(PyFetchExclude::from)
            .collect())
    }

    #[setter]
    fn set_fetch_excludes(&self, excludes: Vec<PyFetchExclude>) -> PyResult<()> {
        let excludes = excludes.iter()
            .map(PyFetchExclude::to_rust)
            .collect::<PyResult<Vec<_>>>()?;
        let mut guard = self.0.lock()?;
        let inst = guard.mojang_mut();
        inst.clear_fetch_exclude();
        for exclude in excludes {
            inst.add_fetch_exclude(exclude);
        }
        Ok(())
    }

    fn add_fetch_exclude(&self, exclude: PyFetchExclude) -> PyResult<()> {
        let exclude = exclude.to_rust()?;
        self.0.lock()?.mojang_mut().add_fetch_exclude(exclude);
        Ok(())
    }

    fn clear_fetch_exclude(&self) -> PyResult<()> {
        self.0.lock()?.mojang_mut().clear_fetch_exclude();
        Ok(())
    }

    #[getter]
    fn demo(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().demo())
    }

    #[setter]
    fn set_demo(&self, demo: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_demo(demo);
        Ok(())
    }

    #[getter]
    fn quick_play(&self) -> PyResult<Option<PyQuickPlay>> {
        Ok(self.0.lock()?.mojang().quick_play().map(|m| match m {
            QuickPlay::Path { path } => PyQuickPlay::Path { path: path.clone() },
            QuickPlay::Singleplayer { name } => PyQuickPlay::Singleplayer { name: name.clone() },
            QuickPlay::Multiplayer { host, port } => PyQuickPlay::Multiplayer { host: host.clone(), port: *port },
            QuickPlay::Realms { id } => PyQuickPlay::Realms { id: id.clone() },
        }))
    }

    #[setter]
    fn set_quick_play(&self, quick_play: Option<PyQuickPlay>) -> PyResult<()> {
        let mut guard = self.0.lock()?;
        match quick_play {
            None => {
                guard.mojang_mut().remove_quick_play();
            }
            Some(quick_play) => {
                guard.mojang_mut().set_quick_play(match quick_play {
                    PyQuickPlay::Path { path } => QuickPlay::Path { path },
                    PyQuickPlay::Singleplayer { name } => QuickPlay::Singleplayer { name },
                    PyQuickPlay::Multiplayer { host, port } => QuickPlay::Multiplayer { host, port },
                    PyQuickPlay::Realms { id } => QuickPlay::Realms { id },
                });
            }
        }
        Ok(())
    }

    #[getter]
    fn resolution(&self) -> PyResult<Option<(u16, u16)>> {
        Ok(self.0.lock()?.mojang().resolution())
    }

    #[setter]
    fn set_resolution(&self, resolution: Option<(u16, u16)>) -> PyResult<()> {
        let mut guard = self.0.lock()?;
        match resolution {
            Some((width, height)) => {
                guard.mojang_mut().set_resolution(width, height);
            }
            None => {
                guard.mojang_mut().remove_resolution();
            }
        }
        Ok(())
    }

    #[getter]
    fn disable_multiplayer(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().disable_multiplayer())
    }

    #[setter]
    fn set_disable_multiplayer(&self, disable_multiplayer: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_disable_multiplayer(disable_multiplayer);
        Ok(())
    }

    #[getter]
    fn disable_chat(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().disable_chat())
    }

    #[setter]
    fn set_disable_chat(&self, disable_chat: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_disable_chat(disable_chat);
        Ok(())
    }

    #[getter]
    fn auth_uuid(&self) -> PyResult<PyUuid> {
        Ok(self.0.lock()?.mojang().auth_uuid().into())
    }

    #[getter]
    fn auth_username(&self) -> PyResult<String> {
        Ok(self.0.lock()?.mojang().auth_username().to_string())
    }

    fn set_auth_offline(&self, uuid: PyUuid, username: String) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_offline(uuid.into(), username);
        Ok(())
    }

    fn set_auth_offline_uuid(&self, uuid: PyUuid) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_offline_uuid(uuid.into());
        Ok(())
    }

    fn set_auth_offline_username(&self, username: String) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_offline_username(username);
        Ok(())
    }

    fn set_auth_offline_username_legacy(&self, username: String) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_offline_username_legacy(username);
        Ok(())
    }

    fn set_auth_offline_hostname(&self) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_offline_hostname();
        Ok(())
    }

    fn set_auth_msa(&self, account: PyRef<'_, msa::PyAccount>) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_auth_msa(&account.0);
        Ok(())
    }
    
    #[getter]
    fn client_id(&self) -> PyResult<String> {
        Ok(self.0.lock()?.mojang().client_id().to_string())
    }

    #[setter]
    fn set_client_id(&self, client_id: String) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_client_id(client_id);
        Ok(())
    }

    #[getter]
    fn fix_legacy_quick_play(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().fix_legacy_quick_play())
    }

    #[setter]
    fn set_fix_legacy_quick_play(&self, fix: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_fix_legacy_quick_play(fix);
        Ok(())
    }

    #[getter]
    fn fix_legacy_proxy(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().fix_legacy_proxy())
    }

    #[setter]
    fn set_fix_legacy_proxy(&self, fix: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_fix_legacy_proxy(fix);
        Ok(())
    }

    #[getter]
    fn fix_legacy_merge_sort(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().fix_legacy_merge_sort())
    }

    #[setter]
    fn set_fix_legacy_merge_sort(&self, fix: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_fix_legacy_merge_sort(fix);
        Ok(())
    }

    #[getter]
    fn fix_legacy_resolution(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().fix_legacy_resolution())
    }

    #[setter]
    fn set_fix_legacy_resolution(&self, fix: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_fix_legacy_resolution(fix);
        Ok(())
    }

    #[getter]
    fn fix_broken_authlib(&self) -> PyResult<bool> {
        Ok(self.0.lock()?.mojang().fix_broken_authlib())
    }

    #[setter]
    fn set_fix_broken_authlib(&self, fix: bool) -> PyResult<()> {
        self.0.lock()?.mojang_mut().set_fix_broken_authlib(fix);
        Ok(())
    }

    #[getter]
    fn fix_lwjgl(&self) -> PyResult<Option<String>> {
        Ok(self.0.lock()?.mojang().fix_lwjgl().map(str::to_string))
    }

    #[setter]
    fn set_fix_lwjgl(&self, lwjgl_version: Option<String>) -> PyResult<()> {
        let mut guard = self.0.lock()?;
        match lwjgl_version {
            Some(lwjgl_version) => {
                guard.mojang_mut().set_fix_lwjgl(lwjgl_version);
            }
            None => {
                guard.mojang_mut().remove_fix_lwjgl();
            }
        }
        Ok(())
    }

    #[pyo3(signature = (handler = None))]
    fn install(&self, py: Python<'_>, handler: Option<Py<PyAny>>) -> PyResult<PyGame> {
        let mut guard = self.0.lock_install()?;
        let inst = guard.mojang_mut();
        handler::install(py, handler, |h| inst.install(h), err::from_mojang)
    }

}

/// Handler for Mojang installer events, every method does nothing by default and can 
/// be overridden by subclasses.
#[pyclass(name = "Handler", module = "portablemc.mojang", frozen, subclass, extends = crate::base::PyHandler)]
pub struct PyHandler;

#[pymethods]
#[allow(unused_variables)]
impl PyHandler {

    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn __new__(args: &Bound<'_, PyAny>, kwargs: Option<&Bound<'_, PyAny>>) -> PyClassInitializer<Self> {
        PyClassInitializer::from(crate::base::PyHandler).add_subclass(Self)
    }

    fn invalidated_version(&self, version: &Bound<'_, PyAny>) {}
    fn fetch_version(&self, version: &Bound<'_, PyAny>) {}
    fn fetched_version(&self, version: &Bound<'_, PyAny>) {}
    fn fixed_legacy_quick_play(&self) {}
    fn fixed_legacy_proxy(&self, host: &Bound<'_, PyAny>, port: &Bound<'_, PyAny>) {}
    fn fixed_legacy_merge_sort(&self) {}
    fn fixed_legacy_resolution(&self) {}
    fn fixed_broken_authlib(&self) {}
    fn warn_unsupported_quick_play(&self) {}
    fn warn_unsupported_resolution(&self) {}

}

/// The Mojang versions manifest.
#[pyclass(name = "Manifest", module = "portablemc.mojang", frozen)]
pub struct PyManifest(Arc<Manifest>);

#[pymethods]
impl PyManifest {

    /// Request the manifest, the handler's `download_progress` method is called if 
    /// the manifest needs to be downloaded.
    #[staticmethod]
    #[pyo3(signature = (handler = None))]
    fn request(py: Python<'_>, handler: Option<Py<PyAny>>) -> PyResult<Self> {
        let mut adapter = HandlerAdapter::new(handler);
        let res = py.detach(|| Manifest::request(&mut adapter));
        adapter.finish()?;
        res.map(|manifest| Self(Arc::new(manifest)))
            .map_err(|e| err::from_mojang(py, e))
    }

    fn __repr__(&self) -> String {
        format!("<portablemc.mojang.Manifest latest_release_name={:?} latest_snapshot_name={:?}>", 
            self.0.latest_release_name(),
            self.0.latest_snapshot_name())
    }

    fn __len__(&self) -> usize {
        self.0.iter().count()
    }

    fn __getitem__(&self, index: isize) -> PyResult<PyManifestVersion> {
        let len = self.__len__() as isize;
        let index = if index < 0 { index + len } else { index };
        if index < 0 || index >= len {
            return Err(PyIndexError::new_err("manifest index out of range"));
        }
        Ok(PyManifestVersion { manifest: Arc::clone(&self.0), index: index as usize })
    }

    fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        let versions = (0..self.__len__())
            .map(|index| PyManifestVersion { manifest: Arc::clone(&self.0), index });
        PyList::new(py, versions)?.try_iter()
    }

    #[getter]
    fn latest_release_name(&self) -> &str {
        self.0.latest_release_name()
    }

    #[getter]
    fn latest_snapshot_name(&self) -> &str {
        self.0.latest_snapshot_name()
    }

    fn find_by_name(&self, name: &str) -> Option<PyManifestVersion> {
        self.0.find_index_of_name(name)
            .map(|index| PyManifestVersion { manifest: Arc::clone(&self.0), index })
    }

}

/// A version in the Mojang versions manifest.
#[pyclass(name = "ManifestVersion", module = "portablemc.mojang", frozen)]
pub struct PyManifestVersion {
    manifest: Arc<Manifest>,
    index: usize,
}

impl PyManifestVersion {

    fn get(&self) -> ManifestVersion<'_> {
        self.manifest.find_by_index(self.index).unwrap()
    }

}

#[pymethods]
impl PyManifestVersion {

    fn __repr__(&self) -> String {
        format!("<portablemc.mojang.ManifestVersion name={:?}>", self.get().name())
    }

    #[getter]
    fn name(&self) -> &str {
        self.get().name()
    }

    #[getter]
    fn channel(&self) -> PyVersionChannel {
        self.get().channel().into()
    }

    #[getter]
    fn time(&self) -> DateTime<FixedOffset> {
        *self.get().time()
    }

    #[getter]
    fn release_time(&self) -> DateTime<FixedOffset> {
        *self.get().release_time()
    }

    #[getter]
    fn url(&self) -> &str {
        self.get().url()
    }

    #[getter]
    fn size(&self) -> Option<u32> {
        self.get().size()
    }

    #[getter]
    fn sha1<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.get().sha1().map(|sha1| PyBytes::new(py, sha1))
    }

}

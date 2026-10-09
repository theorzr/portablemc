//! Adapter forwarding installation events to a Python handler object.

use std::collections::HashSet;
use std::path::PathBuf;

use pyo3::types::{PyList, PySet};
use pyo3::call::PyCallArgs;
use pyo3::prelude::*;

use portablemc::{base, moj, fabric, forge, download};

use crate::base::{PyGame, PyLoadedLibrary, PyLoadedVersion};


/// Run the given installation function with the GIL released, events are forwarded to
/// the optional Python handler. If the handler raised an exception, it is raised in
/// place of the installation result.
pub fn install<E, F, M>(py: Python<'_>, handler: Option<Py<PyAny>>, func: F, map_err: M) -> PyResult<PyGame>
where
    E: Send,
    F: FnOnce(&mut HandlerAdapter) -> Result<base::Game, E> + Send,
    M: FnOnce(Python<'_>, E) -> PyErr,
{
    let mut adapter = HandlerAdapter::new(handler);
    let res = py.detach(|| func(&mut adapter));
    adapter.finish()?;
    let game = res.map_err(|e| map_err(py, e))?;
    PyGame::new(py, game)
}

/// A Rust handler that calls the method named after each event on a Python object.
///
/// Exceptions raised by the handler can't interrupt the installation directly, so the
/// first one is kept, no more methods are called after it, and the next resources
/// download is cancelled to abort the installation as soon as possible.
pub struct HandlerAdapter {
    handler: Option<Py<PyAny>>,
    error: Option<PyErr>,
}

impl HandlerAdapter {

    pub fn new(handler: Option<Py<PyAny>>) -> Self {
        Self { handler, error: None }
    }

    /// Return the exception raised by the handler, if any.
    pub fn finish(self) -> PyResult<()> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Call the given method on the handler, returning its result if successful.
    fn call<'py, A>(&mut self, py: Python<'py>, name: &str, args: A) -> Option<Bound<'py, PyAny>>
    where
        A: PyCallArgs<'py>,
    {

        if self.error.is_some() {
            return None;
        }

        // Also check for signals (such as Ctrl-C) because the GIL is released for the
        // whole installation, so this is the only time the interpreter can handle them.
        let res = py.check_signals()
            .and_then(|()| match &self.handler {
                Some(handler) => handler.bind(py).call_method1(name, args).map(Some),
                None => Ok(None),
            });

        match res {
            Ok(ret) => ret,
            Err(err) => {
                self.error = Some(err);
                None
            }
        }

    }

    /// Call the given method and return true if the returned value is truthy.
    fn call_bool<'py, A>(&mut self, py: Python<'py>, name: &str, args: A) -> bool
    where
        A: PyCallArgs<'py>,
    {
        match self.call(py, name, args).map(|ret| ret.is_truthy()) {
            Some(Ok(ret)) => ret,
            Some(Err(err)) => {
                self.error = Some(err);
                false
            }
            None => false,
        }
    }

    /// Keep the error of a fallible operation done on the handler's arguments.
    fn check<T>(&mut self, res: PyResult<T>) -> Option<T> {
        match res {
            Ok(val) => Some(val),
            Err(err) => {
                if self.error.is_none() {
                    self.error = Some(err);
                }
                None
            }
        }
    }

    fn on_base(&mut self, py: Python<'_>, event: base::Event) {
        use base::Event;
        match event {
            Event::FilterFeatures { features } => {
                let Some(set) = self.check(PySet::new(py, features.iter())) else { return };
                if self.call(py, "filter_features", (&set,)).is_some() {
                    if let Some(new_features) = self.check(set.extract::<HashSet<String>>()) {
                        *features = new_features;
                    }
                }
            }
            Event::LoadedFeatures { features } => {
                let Some(set) = self.check(PySet::new(py, features.iter())) else { return };
                self.call(py, "loaded_features", (set,));
            }
            Event::LoadHierarchy { root_version } => {
                self.call(py, "load_hierarchy", (root_version,));
            }
            Event::LoadedHierarchy { hierarchy } => {
                let hierarchy = hierarchy.iter().cloned().map(PyLoadedVersion);
                let Some(list) = self.check(PyList::new(py, hierarchy)) else { return };
                self.call(py, "loaded_hierarchy", (list,));
            }
            Event::LoadVersion { version, file } => {
                self.call(py, "load_version", (version, file));
            }
            Event::NeedVersion { version, file, retry } => {
                // The inner installer may already have fetched the version.
                if self.call_bool(py, "need_version", (version, file)) {
                    *retry = true;
                }
            }
            Event::LoadedVersion { version, file } => {
                self.call(py, "loaded_version", (version, file));
            }
            Event::LoadClient => {
                self.call(py, "load_client", ());
            }
            Event::LoadedClient { file } => {
                self.call(py, "loaded_client", (file,));
            }
            Event::LoadLibraries => {
                self.call(py, "load_libraries", ());
            }
            Event::FilterLibraries { libraries } => {
                let iter = libraries.iter().cloned().map(PyLoadedLibrary);
                let Some(list) = self.check(PyList::new(py, iter)) else { return };
                if self.call(py, "filter_libraries", (&list,)).is_some() {
                    let new_libraries = list.iter()
                        .map(|lib| lib.cast::<PyLoadedLibrary>()
                            .map(|lib| lib.borrow().0.clone())
                            .map_err(PyErr::from))
                        .collect::<PyResult<Vec<_>>>();
                    if let Some(new_libraries) = self.check(new_libraries) {
                        *libraries = new_libraries;
                    }
                }
            }
            Event::LoadedLibraries { libraries } => {
                let iter = libraries.iter().cloned().map(PyLoadedLibrary);
                let Some(list) = self.check(PyList::new(py, iter)) else { return };
                self.call(py, "loaded_libraries", (list,));
            }
            Event::FilterLibrariesFiles { class_files, natives_files } => {
                let Some(class_list) = self.check(PyList::new(py, class_files.iter())) else { return };
                let Some(natives_list) = self.check(PyList::new(py, natives_files.iter())) else { return };
                if self.call(py, "filter_libraries_files", (&class_list, &natives_list)).is_some() {
                    if let Some(new_class_files) = self.check(class_list.extract::<Vec<PathBuf>>()) {
                        *class_files = new_class_files;
                    }
                    if let Some(new_natives_files) = self.check(natives_list.extract::<Vec<PathBuf>>()) {
                        *natives_files = new_natives_files;
                    }
                }
            }
            Event::LoadedLibrariesFiles { class_files, natives_files } => {
                let Some(class_list) = self.check(PyList::new(py, class_files.iter())) else { return };
                let Some(natives_list) = self.check(PyList::new(py, natives_files.iter())) else { return };
                self.call(py, "loaded_libraries_files", (class_list, natives_list));
            }
            Event::NoLogger => {
                self.call(py, "no_logger", ());
            }
            Event::LoadLogger { id } => {
                self.call(py, "load_logger", (id,));
            }
            Event::LoadedLogger { id } => {
                self.call(py, "loaded_logger", (id,));
            }
            Event::NoAssets => {
                self.call(py, "no_assets", ());
            }
            Event::LoadAssets { id } => {
                self.call(py, "load_assets", (id,));
            }
            Event::LoadedAssets { id, count } => {
                self.call(py, "loaded_assets", (id, count));
            }
            Event::VerifiedAssets { id, count } => {
                self.call(py, "verified_assets", (id, count));
            }
            Event::LoadJvm { major_version } => {
                self.call(py, "load_jvm", (major_version,));
            }
            Event::FoundJvmSystemVersion { file, version, compatible } => {
                self.call(py, "found_jvm_system_version", (file, version, compatible));
            }
            Event::WarnJvmUnsupportedDynamicCrt => {
                self.call(py, "warn_jvm_unsupported_dynamic_crt", ());
            }
            Event::WarnJvmUnsupportedPlatform => {
                self.call(py, "warn_jvm_unsupported_platform", ());
            }
            Event::WarnJvmMissingDistribution => {
                self.call(py, "warn_jvm_missing_distribution", ());
            }
            Event::LoadedJvm { file, version, compatible } => {
                self.call(py, "loaded_jvm", (file, version, compatible));
            }
            Event::DownloadResources { cancel } => {
                if self.call_bool(py, "download_resources", ()) || self.error.is_some() {
                    *cancel = true;
                }
            }
            Event::DownloadedResources => {
                self.call(py, "downloaded_resources", ());
            }
            Event::DownloadProgress { count, total_count, size, total_size } => {
                self.call(py, "download_progress", (count, total_count, size, total_size));
            }
            Event::ExtractedBinaries { dir } => {
                self.call(py, "extracted_binaries", (dir,));
            }
            _ => {}
        }
    }

    fn on_mojang(&mut self, py: Python<'_>, event: moj::Event) {
        use moj::Event;
        match event {
            Event::Base(event) => self.on_base(py, event),
            Event::InvalidatedVersion { version } => {
                self.call(py, "invalidated_version", (version,));
            }
            Event::FetchVersion { version } => {
                self.call(py, "fetch_version", (version,));
            }
            Event::FetchedVersion { version } => {
                self.call(py, "fetched_version", (version,));
            }
            Event::FixedLegacyQuickPlay => {
                self.call(py, "fixed_legacy_quick_play", ());
            }
            Event::FixedLegacyProxy { host, port } => {
                self.call(py, "fixed_legacy_proxy", (host, port));
            }
            Event::FixedLegacyMergeSort => {
                self.call(py, "fixed_legacy_merge_sort", ());
            }
            Event::FixedLegacyResolution => {
                self.call(py, "fixed_legacy_resolution", ());
            }
            Event::FixedBrokenAuthlib => {
                self.call(py, "fixed_broken_authlib", ());
            }
            Event::WarnUnsupportedQuickPlay => {
                self.call(py, "warn_unsupported_quick_play", ());
            }
            Event::WarnUnsupportedResolution => {
                self.call(py, "warn_unsupported_resolution", ());
            }
            _ => {}
        }
    }

    fn on_fabric(&mut self, py: Python<'_>, event: fabric::Event) {
        use fabric::Event;
        match event {
            Event::Mojang(event) => self.on_mojang(py, event),
            Event::FetchVersion { game_version, loader_version } => {
                self.call(py, "fetch_loader_version", (game_version, loader_version));
            }
            Event::FetchedVersion { game_version, loader_version } => {
                self.call(py, "fetched_loader_version", (game_version, loader_version));
            }
            _ => {}
        }
    }

    fn on_forge(&mut self, py: Python<'_>, event: forge::Event) {
        use forge::Event;
        match event {
            Event::Mojang(event) => self.on_mojang(py, event),
            Event::Installing { tmp_dir, reason } => {
                self.call(py, "installing", (tmp_dir, crate::forge::PyInstallReason::from(reason)));
            }
            Event::FetchInstaller { version } => {
                self.call(py, "fetch_installer", (version,));
            }
            Event::FetchedInstaller { version } => {
                self.call(py, "fetched_installer", (version,));
            }
            Event::InstallingGame => {
                self.call(py, "installing_game", ());
            }
            Event::FetchInstallerLibraries => {
                self.call(py, "fetch_installer_libraries", ());
            }
            Event::FetchedInstallerLibraries => {
                self.call(py, "fetched_installer_libraries", ());
            }
            Event::RunInstallerProcessor { name, task } => {
                self.call(py, "run_installer_processor", (name.as_str(), task));
            }
            Event::Installed => {
                self.call(py, "installed", ());
            }
            _ => {}
        }
    }

}

impl base::Handler for HandlerAdapter {
    fn on_event(&mut self, event: base::Event) {
        Python::attach(|py| self.on_base(py, event));
    }
}

impl moj::Handler for HandlerAdapter {
    fn on_event(&mut self, event: moj::Event) {
        Python::attach(|py| self.on_mojang(py, event));
    }
}

impl fabric::Handler for HandlerAdapter {
    fn on_event(&mut self, event: fabric::Event) {
        Python::attach(|py| self.on_fabric(py, event));
    }
}

impl forge::Handler for HandlerAdapter {
    fn on_event(&mut self, event: forge::Event) {
        Python::attach(|py| self.on_forge(py, event));
    }
}

impl download::Handler for HandlerAdapter {
    fn on_progress(&mut self, count: u32, total_count: u32, size: u32, total_size: u32) {
        Python::attach(|py| {
            self.call(py, "download_progress", (count, total_count, size, total_size));
        });
    }
}

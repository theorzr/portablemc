//! Python exception types and conversion from the library's error types.
//!
//! Nested errors (a Fabric error wrapping a Mojang error wrapping a base error) are
//! flattened, so the raised exception is always the most specific one, and all
//! installer exceptions inherit from `portablemc.base.Error`.

use std::error::Error as StdError;
use std::io;

use pyo3::exceptions::PyException;
use pyo3::types::PyBytes;
use pyo3::{create_exception, prelude::*, IntoPyObjectExt, PyTypeInfo};

use portablemc::{base, moj, fabric, forge, msa};


create_exception!(portablemc.base, Error, PyException, "Base class for all installer errors.");
create_exception!(portablemc.base, HierarchyLoopError, Error);
create_exception!(portablemc.base, VersionNotFoundError, Error);
create_exception!(portablemc.base, AssetsNotFoundError, Error);
create_exception!(portablemc.base, ClientNotFoundError, Error);
create_exception!(portablemc.base, LibraryNotFoundError, Error);
create_exception!(portablemc.base, JvmNotFoundError, Error);
create_exception!(portablemc.base, MainClassNotFoundError, Error);
create_exception!(portablemc.base, DownloadResourcesCancelledError, Error);
create_exception!(portablemc.base, DownloadError, Error);
create_exception!(portablemc.base, InternalError, Error);

create_exception!(portablemc.mojang, LwjglFixNotFoundError, Error);

/// Fabric and Forge both have a `LatestVersionNotFoundError`.
pub mod fabric_exc {
    pyo3::create_exception!(portablemc.fabric, LatestVersionNotFoundError, super::Error);
}
create_exception!(portablemc.fabric, GameVersionNotFoundError, Error);
create_exception!(portablemc.fabric, LoaderVersionNotFoundError, Error);

pub mod forge_exc {
    pyo3::create_exception!(portablemc.forge, LatestVersionNotFoundError, super::Error);
}
create_exception!(portablemc.forge, InstallerNotFoundError, Error);
create_exception!(portablemc.forge, MavenMetadataMalformedError, Error);
create_exception!(portablemc.forge, InstallerProfileNotFoundError, Error);
create_exception!(portablemc.forge, InstallerProfileIncoherentError, Error);
create_exception!(portablemc.forge, InstallerVersionMetadataNotFoundError, Error);
create_exception!(portablemc.forge, InstallerFileNotFoundError, Error);
create_exception!(portablemc.forge, InstallerProcessorNotFoundError, Error);
create_exception!(portablemc.forge, InstallerProcessorMainClassNotFoundError, Error);
create_exception!(portablemc.forge, InstallerProcessorDependencyNotFoundError, Error);
create_exception!(portablemc.forge, InstallerProcessorFailedError, Error);
create_exception!(portablemc.forge, InstallerProcessorCorruptedError, Error);

create_exception!(portablemc.msa, AuthError, PyException, "Base class for all authentication errors.");
create_exception!(portablemc.msa, AuthDeclinedError, AuthError);
create_exception!(portablemc.msa, AuthTimedOutError, AuthError);
create_exception!(portablemc.msa, AuthOutdatedTokenError, AuthError);
create_exception!(portablemc.msa, AuthDoesNotOwnGameError, AuthError);
create_exception!(portablemc.msa, AuthInvalidStatusError, AuthError);
create_exception!(portablemc.msa, DatabaseError, PyException, "Base class for all account database errors.");
create_exception!(portablemc.msa, DatabaseCorruptedError, DatabaseError);
create_exception!(portablemc.msa, DatabaseWriteFailedError, DatabaseError);


/// Register the base exceptions in the `portablemc.base` module.
pub fn add_base(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("Error", py.get_type::<Error>())?;
    m.add("HierarchyLoopError", py.get_type::<HierarchyLoopError>())?;
    m.add("VersionNotFoundError", py.get_type::<VersionNotFoundError>())?;
    m.add("AssetsNotFoundError", py.get_type::<AssetsNotFoundError>())?;
    m.add("ClientNotFoundError", py.get_type::<ClientNotFoundError>())?;
    m.add("LibraryNotFoundError", py.get_type::<LibraryNotFoundError>())?;
    m.add("JvmNotFoundError", py.get_type::<JvmNotFoundError>())?;
    m.add("MainClassNotFoundError", py.get_type::<MainClassNotFoundError>())?;
    m.add("DownloadResourcesCancelledError", py.get_type::<DownloadResourcesCancelledError>())?;
    m.add("DownloadError", py.get_type::<DownloadError>())?;
    m.add("InternalError", py.get_type::<InternalError>())?;
    Ok(())
}

/// Register the Mojang exceptions in the `portablemc.mojang` module.
pub fn add_mojang(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("LwjglFixNotFoundError", py.get_type::<LwjglFixNotFoundError>())?;
    Ok(())
}

/// Register the Fabric exceptions in the `portablemc.fabric` module.
pub fn add_fabric(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("LatestVersionNotFoundError", py.get_type::<fabric_exc::LatestVersionNotFoundError>())?;
    m.add("GameVersionNotFoundError", py.get_type::<GameVersionNotFoundError>())?;
    m.add("LoaderVersionNotFoundError", py.get_type::<LoaderVersionNotFoundError>())?;
    Ok(())
}

/// Register the Forge exceptions in the `portablemc.forge` module.
pub fn add_forge(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("LatestVersionNotFoundError", py.get_type::<forge_exc::LatestVersionNotFoundError>())?;
    m.add("InstallerNotFoundError", py.get_type::<InstallerNotFoundError>())?;
    m.add("MavenMetadataMalformedError", py.get_type::<MavenMetadataMalformedError>())?;
    m.add("InstallerProfileNotFoundError", py.get_type::<InstallerProfileNotFoundError>())?;
    m.add("InstallerProfileIncoherentError", py.get_type::<InstallerProfileIncoherentError>())?;
    m.add("InstallerVersionMetadataNotFoundError", py.get_type::<InstallerVersionMetadataNotFoundError>())?;
    m.add("InstallerFileNotFoundError", py.get_type::<InstallerFileNotFoundError>())?;
    m.add("InstallerProcessorNotFoundError", py.get_type::<InstallerProcessorNotFoundError>())?;
    m.add("InstallerProcessorMainClassNotFoundError", py.get_type::<InstallerProcessorMainClassNotFoundError>())?;
    m.add("InstallerProcessorDependencyNotFoundError", py.get_type::<InstallerProcessorDependencyNotFoundError>())?;
    m.add("InstallerProcessorFailedError", py.get_type::<InstallerProcessorFailedError>())?;
    m.add("InstallerProcessorCorruptedError", py.get_type::<InstallerProcessorCorruptedError>())?;
    Ok(())
}

/// Register the authentication exceptions in the `portablemc.msa` module.
pub fn add_msa(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("AuthError", py.get_type::<AuthError>())?;
    m.add("AuthDeclinedError", py.get_type::<AuthDeclinedError>())?;
    m.add("AuthTimedOutError", py.get_type::<AuthTimedOutError>())?;
    m.add("AuthOutdatedTokenError", py.get_type::<AuthOutdatedTokenError>())?;
    m.add("AuthDoesNotOwnGameError", py.get_type::<AuthDoesNotOwnGameError>())?;
    m.add("AuthInvalidStatusError", py.get_type::<AuthInvalidStatusError>())?;
    m.add("DatabaseError", py.get_type::<DatabaseError>())?;
    m.add("DatabaseCorruptedError", py.get_type::<DatabaseCorruptedError>())?;
    m.add("DatabaseWriteFailedError", py.get_type::<DatabaseWriteFailedError>())?;
    Ok(())
}

/// Create an exception of the given type with a message and set the given attributes
/// on the exception instance.
fn new_err<'py, T: PyTypeInfo>(py: Python<'py>, message: String, attrs: &[(&str, Bound<'py, PyAny>)]) -> PyErr {
    let err = PyErr::new::<T, _>(message);
    let value = err.value(py);
    for (name, attr) in attrs {
        if let Err(e) = value.setattr(*name, attr) {
            return e;
        }
    }
    err
}

/// Shorthand for converting an attribute value into a Python object.
fn attr<'py, T>(py: Python<'py>, value: T) -> Bound<'py, PyAny>
where
    T: IntoPyObject<'py>,
{
    value.into_bound_py_any(py)
        .unwrap_or_else(|_| py.None().into_bound(py))
}

/// Convert an internal error into an exception that has the most relevant cause, an
/// I/O error becomes an `OSError` cause so that its kind is preserved.
fn internal_cause(py: Python<'_>, err: &PyErr, error: Box<dyn StdError + Send + Sync>) {
    let cause = match error.downcast::<io::Error>() {
        Ok(io_err) => PyErr::from(*io_err),
        Err(error) => PyException::new_err(error.to_string()),
    };
    err.set_cause(py, Some(cause));
}

pub fn from_base(py: Python<'_>, error: base::Error) -> PyErr {
    let message = error.to_string();
    match error {
        base::Error::HierarchyLoop { version } =>
            new_err::<HierarchyLoopError>(py, message, &[("version", attr(py, version))]),
        base::Error::VersionNotFound { version } =>
            new_err::<VersionNotFoundError>(py, message, &[("version", attr(py, version))]),
        base::Error::AssetsNotFound { id } =>
            new_err::<AssetsNotFoundError>(py, message, &[("id", attr(py, id))]),
        base::Error::ClientNotFound {  } =>
            new_err::<ClientNotFoundError>(py, message, &[]),
        base::Error::LibraryNotFound { name } =>
            new_err::<LibraryNotFoundError>(py, message, &[("name", attr(py, name.as_str()))]),
        base::Error::JvmNotFound { major_version } =>
            new_err::<JvmNotFoundError>(py, message, &[("major_version", attr(py, major_version))]),
        base::Error::MainClassNotFound {  } =>
            new_err::<MainClassNotFoundError>(py, message, &[]),
        base::Error::DownloadResourcesCancelled {  } =>
            new_err::<DownloadResourcesCancelledError>(py, message, &[]),
        base::Error::Download { batch } => {
            let errors = batch.iter_errors()
                .map(|e| (e.url().to_string(), e.file().to_path_buf(), e.kind().to_string()))
                .collect::<Vec<_>>();
            new_err::<DownloadError>(py, message, &[
                ("errors", attr(py, errors)),
                ("count", attr(py, batch.len())),
            ])
        }
        base::Error::Internal { error, origin } => {
            let err = new_err::<InternalError>(py, message, &[("origin", attr(py, &*origin))]);
            internal_cause(py, &err, error);
            err
        }
        _ => PyErr::new::<Error, _>(message),
    }
}

pub fn from_mojang(py: Python<'_>, error: moj::Error) -> PyErr {
    let message = error.to_string();
    match error {
        moj::Error::Base(error) => from_base(py, error),
        moj::Error::LwjglFixNotFound { version } =>
            new_err::<LwjglFixNotFoundError>(py, message, &[("version", attr(py, version))]),
        _ => PyErr::new::<Error, _>(message),
    }
}

pub fn from_fabric(py: Python<'_>, error: fabric::Error) -> PyErr {
    let message = error.to_string();
    match error {
        fabric::Error::Mojang(error) => from_mojang(py, error),
        fabric::Error::LatestVersionNotFound { game_version, stable } =>
            new_err::<fabric_exc::LatestVersionNotFoundError>(py, message, &[
                ("game_version", attr(py, game_version)),
                ("stable", attr(py, stable)),
            ]),
        fabric::Error::GameVersionNotFound { game_version } =>
            new_err::<GameVersionNotFoundError>(py, message, &[
                ("game_version", attr(py, game_version)),
            ]),
        fabric::Error::LoaderVersionNotFound { game_version, loader_version } =>
            new_err::<LoaderVersionNotFoundError>(py, message, &[
                ("game_version", attr(py, game_version)),
                ("loader_version", attr(py, loader_version)),
            ]),
        _ => PyErr::new::<Error, _>(message),
    }
}

pub fn from_forge(py: Python<'_>, error: forge::Error) -> PyErr {
    let message = error.to_string();
    match error {
        forge::Error::Mojang(error) => from_mojang(py, error),
        forge::Error::LatestVersionNotFound { game_version, stable } =>
            new_err::<forge_exc::LatestVersionNotFoundError>(py, message, &[
                ("game_version", attr(py, game_version)),
                ("stable", attr(py, stable)),
            ]),
        forge::Error::InstallerNotFound { version } =>
            new_err::<InstallerNotFoundError>(py, message, &[("version", attr(py, version))]),
        forge::Error::MavenMetadataMalformed {  } =>
            new_err::<MavenMetadataMalformedError>(py, message, &[]),
        forge::Error::InstallerProfileNotFound {  } =>
            new_err::<InstallerProfileNotFoundError>(py, message, &[]),
        forge::Error::InstallerProfileIncoherent {  } =>
            new_err::<InstallerProfileIncoherentError>(py, message, &[]),
        forge::Error::InstallerVersionMetadataNotFound {  } =>
            new_err::<InstallerVersionMetadataNotFoundError>(py, message, &[]),
        forge::Error::InstallerFileNotFound { entry } =>
            new_err::<InstallerFileNotFoundError>(py, message, &[("entry", attr(py, entry))]),
        forge::Error::InstallerProcessorNotFound { name } =>
            new_err::<InstallerProcessorNotFoundError>(py, message, &[("name", attr(py, name.as_str()))]),
        forge::Error::InstallerProcessorMainClassNotFound { name } =>
            new_err::<InstallerProcessorMainClassNotFoundError>(py, message, &[("name", attr(py, name.as_str()))]),
        forge::Error::InstallerProcessDependencyNotFound { name, dependency } =>
            new_err::<InstallerProcessorDependencyNotFoundError>(py, message, &[
                ("name", attr(py, name.as_str())),
                ("dependency", attr(py, dependency.as_str())),
            ]),
        forge::Error::InstallerProcessorFailed { name, output } =>
            new_err::<InstallerProcessorFailedError>(py, message, &[
                ("name", attr(py, name.as_str())),
                ("status", attr(py, output.status.code())),
                ("stdout", PyBytes::new(py, &output.stdout).into_any()),
                ("stderr", PyBytes::new(py, &output.stderr).into_any()),
            ]),
        forge::Error::InstallerProcessorCorrupted { name, file, expected_sha1 } =>
            new_err::<InstallerProcessorCorruptedError>(py, message, &[
                ("name", attr(py, name.as_str())),
                ("file", attr(py, &*file)),
                ("expected_sha1", PyBytes::new(py, &*expected_sha1).into_any()),
            ]),
        _ => PyErr::new::<Error, _>(message),
    }
}

pub fn from_auth(py: Python<'_>, error: msa::AuthError) -> PyErr {
    let message = error.to_string();
    match error {
        msa::AuthError::Declined => new_err::<AuthDeclinedError>(py, message, &[]),
        msa::AuthError::TimedOut => new_err::<AuthTimedOutError>(py, message, &[]),
        msa::AuthError::OutdatedToken => new_err::<AuthOutdatedTokenError>(py, message, &[]),
        msa::AuthError::DoesNotOwnGame => new_err::<AuthDoesNotOwnGameError>(py, message, &[]),
        msa::AuthError::InvalidStatus(status) =>
            new_err::<AuthInvalidStatusError>(py, message, &[("status", attr(py, status))]),
        msa::AuthError::Internal(error) => {
            let err = PyErr::new::<AuthError, _>(message);
            internal_cause(py, &err, error);
            err
        }
        _ => PyErr::new::<AuthError, _>(message),
    }
}

pub fn from_database(py: Python<'_>, error: msa::DatabaseError) -> PyErr {
    let message = error.to_string();
    match error {
        msa::DatabaseError::Io(error) => {
            let err = PyErr::new::<DatabaseError, _>(message);
            err.set_cause(py, Some(PyErr::from(error)));
            err
        }
        msa::DatabaseError::Corrupted => new_err::<DatabaseCorruptedError>(py, message, &[]),
        msa::DatabaseError::WriteFailed => new_err::<DatabaseWriteFailedError>(py, message, &[]),
        _ => PyErr::new::<DatabaseError, _>(message),
    }
}

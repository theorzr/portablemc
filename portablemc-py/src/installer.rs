//! Generic installer data that is shared between all kind of installers, this is used
//! to allow inheritance.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};
use std::thread::{self, ThreadId};
use std::ops::{Deref, DerefMut};

use pyo3::exceptions::PyRuntimeError;
use pyo3::sync::MutexExt;
use pyo3::prelude::*;

use portablemc::{base, moj, fabric, forge};


/// An installer shared between all the Python classes of an installer hierarchy.
///
/// The lock is kept for the whole installation while the GIL is released, so other
/// threads wait for the installation to finish, with the GIL released to not deadlock
/// with the installation's handler. The handler itself runs on the installing thread,
/// so it gets an error when accessing the installer instead of deadlocking.
pub struct SharedInstaller {
    inner: Mutex<GenericInstaller>,
    install_thread: Mutex<Option<ThreadId>>,
}

impl SharedInstaller {

    pub fn new(inst: GenericInstaller) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(inst),
            install_thread: Mutex::new(None),
        })
    }

    /// Lock the installer, waiting for any installation on another thread to finish.
    pub fn lock(&self) -> PyResult<MutexGuard<'_, GenericInstaller>> {

        match self.inner.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(err)) => return Ok(err.into_inner()),
            Err(TryLockError::WouldBlock) => {}
        }

        if *self.install_thread.lock().unwrap() == Some(thread::current().id()) {
            return Err(PyRuntimeError::new_err("installer cannot be accessed while it's installing"));
        }

        Ok(Python::attach(|py| self.inner.lock_py_attached(py))
            .unwrap_or_else(PoisonError::into_inner))

    }

    /// Lock the installer for an installation on the current thread.
    pub fn lock_install(&self) -> PyResult<InstallGuard<'_>> {
        let guard = self.lock()?;
        *self.install_thread.lock().unwrap() = Some(thread::current().id());
        Ok(InstallGuard { shared: self, guard })
    }

}

/// The installer lock held during an installation.
pub struct InstallGuard<'a> {
    shared: &'a SharedInstaller,
    guard: MutexGuard<'a, GenericInstaller>,
}

impl Deref for InstallGuard<'_> {
    type Target = GenericInstaller;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl DerefMut for InstallGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl Drop for InstallGuard<'_> {
    fn drop(&mut self) {
        // The guard field is dropped after this, so the thread is cleared before unlock.
        *self.shared.install_thread.lock().unwrap() = None;
    }
}

/// A generic installer of any kind, giving access to its inner installers.
#[derive(Debug)]
pub enum GenericInstaller {
    Base(base::Installer),
    Mojang(moj::Installer),
    Fabric(fabric::Installer),
    Forge(forge::Installer),
}

impl GenericInstaller {

    pub fn base(&self) -> &base::Installer {
        match self {
            GenericInstaller::Base(installer) => installer,
            GenericInstaller::Mojang(installer) => installer.base(),
            GenericInstaller::Fabric(installer) => installer.mojang().base(),
            GenericInstaller::Forge(installer) => installer.mojang().base(),
        }
    }

    pub fn base_mut(&mut self) -> &mut base::Installer {
        match self {
            GenericInstaller::Base(installer) => installer,
            GenericInstaller::Mojang(installer) => installer.base_mut(),
            GenericInstaller::Fabric(installer) => installer.mojang_mut().base_mut(),
            GenericInstaller::Forge(installer) => installer.mojang_mut().base_mut(),
        }
    }

    pub fn mojang(&self) -> &moj::Installer {
        match self {
            GenericInstaller::Base(_) => panic!("not a mojang installer"),
            GenericInstaller::Mojang(installer) => installer,
            GenericInstaller::Fabric(installer) => installer.mojang(),
            GenericInstaller::Forge(installer) => installer.mojang(),
        }
    }

    pub fn mojang_mut(&mut self) -> &mut moj::Installer {
        match self {
            GenericInstaller::Base(_) => panic!("not a mojang installer"),
            GenericInstaller::Mojang(installer) => installer,
            GenericInstaller::Fabric(installer) => installer.mojang_mut(),
            GenericInstaller::Forge(installer) => installer.mojang_mut(),
        }
    }

    pub fn fabric(&self) -> &fabric::Installer {
        match self {
            GenericInstaller::Fabric(installer) => installer,
            _ => panic!("not a fabric installer"),
        }
    }

    pub fn fabric_mut(&mut self) -> &mut fabric::Installer {
        match self {
            GenericInstaller::Fabric(installer) => installer,
            _ => panic!("not a fabric installer"),
        }
    }

    pub fn forge(&self) -> &forge::Installer {
        match self {
            GenericInstaller::Forge(installer) => installer,
            _ => panic!("not a forge installer"),
        }
    }

    pub fn forge_mut(&mut self) -> &mut forge::Installer {
        match self {
            GenericInstaller::Forge(installer) => installer,
            _ => panic!("not a forge installer"),
        }
    }
    
}

from functools import partial
from typing_extensions import Self
from os import PathLike
from pathlib import Path
from enum import Enum, auto
from subprocess import Popen


class Installer:

    def __new__(cls, version: str) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def version(self) -> str: ...
    @version.setter
    def version(self, version: str): ...

    @property
    def versions_dir(self) -> Path: ...
    @versions_dir.setter
    def versions_dir(self, dir: str | PathLike[str]): ...

    @property
    def libraries_dir(self) -> Path: ...
    @libraries_dir.setter
    def libraries_dir(self, dir: str | PathLike[str]): ...

    @property
    def assets_dir(self) -> Path: ...
    @assets_dir.setter
    def assets_dir(self, dir: str | PathLike[str]): ...

    @property
    def jvm_dir(self) -> Path: ...
    @jvm_dir.setter
    def jvm_dir(self, dir: str | PathLike[str]): ...

    @property
    def bin_dir(self) -> Path: ...
    @bin_dir.setter
    def bin_dir(self, dir: str | PathLike[str]): ...

    @property
    def mc_dir(self) -> Path: ...
    @mc_dir.setter
    def mc_dir(self, dir: str | PathLike[str]): ...

    # Not a property because it sets all others dirs!
    def set_main_dir(self, dir: str | PathLike[str]) -> None: ...

    @property
    def strict_assets_check(self) -> bool: ...
    @strict_assets_check.setter
    def strict_assets_check(self, strict: bool): ...

    @property
    def strict_libraries_check(self) -> bool: ...
    @strict_libraries_check.setter
    def strict_libraries_check(self, strict: bool): ...

    @property
    def strict_jvm_check(self) -> bool: ...
    @strict_jvm_check.setter
    def strict_jvm_check(self, strict: bool): ...

    @property
    def jvm_policy(self) -> Path | JvmPolicy: ...
    @jvm_policy.setter
    def jvm_policy(self, policy: str | PathLike[str] | JvmPolicy): ...

    @property
    def launcher_name(self) -> str: ...
    @launcher_name.setter
    def launcher_name(self, name: str): ...

    @property
    def launcher_version(self) -> str: ...
    @launcher_version.setter
    def launcher_version(self, name: str): ...

    def install(self, handler: Handler | None = None) -> Game:
        """Install the version and return the game ready to be launched. The GIL is
        released during the installation, the handler's methods are called from the
        calling thread. If the handler raises an exception, no other handler method
        is called, the installation is aborted as soon as possible and the exception
        is raised from this method."""


class JvmPolicy(Enum):
    System = auto()
    Mojang = auto()
    SystemThenMojang = auto()
    MojangThenSystem = auto()


class Handler:
    """Handler for installation events, every method does nothing by default and can be
    overridden by subclasses. Methods are called by name, so any object with the
    relevant methods can be used as a handler."""

    def filter_features(self, features: set[str]) -> None:
        """Filter features used to resolve version rules, the set can be modified."""
    def loaded_features(self, features: set[str]) -> None: ...
    def load_hierarchy(self, root_version: str) -> None: ...
    def loaded_hierarchy(self, hierarchy: list[LoadedVersion]) -> None: ...
    def load_version(self, version: str, file: Path) -> None: ...
    def need_version(self, version: str, file: Path) -> bool:
        """The version metadata file is missing, the handler can install it and return
        true to retry loading it."""
    def loaded_version(self, version: str, file: Path) -> None: ...
    def load_client(self) -> None: ...
    def loaded_client(self, file: Path) -> None: ...
    def load_libraries(self) -> None: ...
    def filter_libraries(self, libraries: list[LoadedLibrary]) -> None:
        """Filter libraries before their verification, the list can be modified."""
    def loaded_libraries(self, libraries: list[LoadedLibrary]) -> None: ...
    def filter_libraries_files(self, class_files: list[Path], natives_files: list[Path]) -> None:
        """Filter the verified libraries files, the lists can be modified."""
    def loaded_libraries_files(self, class_files: list[Path], natives_files: list[Path]) -> None: ...
    def no_logger(self) -> None: ...
    def load_logger(self, id: str) -> None: ...
    def loaded_logger(self, id: str) -> None: ...
    def no_assets(self) -> None: ...
    def load_assets(self, id: str) -> None: ...
    def loaded_assets(self, id: str, count: int) -> None: ...
    def verified_assets(self, id: str, count: int) -> None: ...
    def load_jvm(self, major_version: int) -> None: ...
    def found_jvm_system_version(self, file: Path, version: str, compatible: bool) -> None: ...
    def warn_jvm_unsupported_dynamic_crt(self) -> None: ...
    def warn_jvm_unsupported_platform(self) -> None: ...
    def warn_jvm_missing_distribution(self) -> None: ...
    def loaded_jvm(self, file: Path, version: str | None, compatible: bool) -> None: ...
    def download_resources(self) -> bool:
        """Resources will be downloaded, return true to cancel the download, the
        installation then fails with `DownloadResourcesCancelledError`."""
    def downloaded_resources(self) -> None: ...
    def download_progress(self, count: int, total_count: int, size: int, total_size: int) -> None: ...
    def extracted_binaries(self, dir: Path) -> None: ...


class VersionChannel(Enum):
    Release = auto()
    Snapshot = auto()
    Beta = auto()
    Alpha = auto()


class LoadedVersion:

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def dir(self) -> Path: ...
    @property
    def channel(self) -> VersionChannel | None: ...


class LoadedLibrary:

    def __new__(cls, name: str, path: str | PathLike[str] | None = None, download: LibraryDownload | None = None, natives: bool = False) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str:
        """The library maven specifier, 'group:artifact:version[:classifier][@extension]'."""
    @name.setter
    def name(self, name: str): ...

    @property
    def path(self) -> Path | None:
        """The path relative to the libraries directory, derived from the name if none."""
    @path.setter
    def path(self, path: str | PathLike[str] | None): ...

    @property
    def download(self) -> LibraryDownload | None: ...
    @download.setter
    def download(self, download: LibraryDownload | None): ...

    @property
    def natives(self) -> bool: ...
    @natives.setter
    def natives(self, natives: bool): ...


class LibraryDownload:

    def __new__(cls, url: str, size: int | None = None, sha1: bytes | None = None) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def url(self) -> str: ...
    @property
    def size(self) -> int | None: ...
    @property
    def sha1(self) -> bytes | None: ...


class Game:

    def __repr__(self) -> str: ...

    jvm_file: Path
    mc_dir: Path
    main_class: str
    jvm_args: list[str]
    game_args: list[str]

    def args(self) -> list[str | Path]:
        """Return the full command line, starting with the JVM executable."""

    def command(self) -> partial[Popen[bytes]]:
        """Return a partial `subprocess.Popen` constructor with the command line and
        working directory already set, further arguments are given to `Popen`."""


def default_main_dir() -> Path | None: ...


class Error(Exception):
    """Base class for all installer errors."""

class HierarchyLoopError(Error):
    version: str

class VersionNotFoundError(Error):
    version: str

class AssetsNotFoundError(Error):
    id: str

class ClientNotFoundError(Error): ...

class LibraryNotFoundError(Error):
    name: str

class JvmNotFoundError(Error):
    major_version: int

class MainClassNotFoundError(Error): ...

class DownloadResourcesCancelledError(Error): ...

class DownloadError(Error):
    errors: list[tuple[str, Path, str]]
    """The (url, file, reason) of each failed entry."""
    count: int
    """Total number of entries in the download."""

class InternalError(Error):
    """Unexpected internal error, the cause is set to an `OSError` for I/O errors."""
    origin: str

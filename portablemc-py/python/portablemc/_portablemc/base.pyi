from functools import partial
from typing_extensions import Self
from os import PathLike
from pathlib import Path
from enum import Enum, auto
from subprocess import Popen


class Installer:
    """The installer that supports the minimal basic format for version metadata, with
    automatic installation of libraries, assets, loggers and JVM. It doesn't fetch
    missing versions, enables no feature and provides no fix for legacy versions, most
    of the time you want to use `moj.Installer` instead."""

    def __new__(cls, version: str) -> Self:
        """Create a new installer for the given root version, directories that are not
        configured are derived from the default main directory."""

    def __repr__(self) -> str: ...

    @property
    def version(self) -> str:
        """The root version to load with its hierarchy and install."""
    @version.setter
    def version(self, version: str): ...

    @property
    def versions_dir(self) -> Path:
        """The directory where versions are stored."""
    @versions_dir.setter
    def versions_dir(self, dir: str | PathLike[str]): ...

    @property
    def libraries_dir(self) -> Path:
        """The directory where libraries are stored, organized like a maven repository."""
    @libraries_dir.setter
    def libraries_dir(self, dir: str | PathLike[str]): ...

    @property
    def assets_dir(self) -> Path:
        """The directory where assets, assets index, cached skins and logger configs are
        stored. The game caches player skins in it, so it needs write permission."""
    @assets_dir.setter
    def assets_dir(self, dir: str | PathLike[str]): ...

    @property
    def jvm_dir(self) -> Path:
        """The directory where Mojang-provided JVMs are installed."""
    @jvm_dir.setter
    def jvm_dir(self, dir: str | PathLike[str]): ...

    @property
    def bin_dir(self) -> Path:
        """The directory where natives (.dll, .so) are extracted before startup, the game
        needs read/write permission to it. A sub-directory unique to the libraries files
        is created in it, it is temporary and can be removed once all instances of the
        game are terminated."""
    @bin_dir.setter
    def bin_dir(self, dir: str | PathLike[str]): ...

    @property
    def mc_dir(self) -> Path:
        """The working directory of the game, where all user data is saved (saves,
        resource packs, options...), the game needs read/write permission to it."""
    @mc_dir.setter
    def mc_dir(self, dir: str | PathLike[str]): ...

    # Not a property because it sets all others dirs!
    def set_main_dir(self, dir: str | PathLike[str]) -> None:
        """Set the `versions`, `assets`, `libraries` and `jvm` directories as children of
        the given directory. On Windows, long NT UNC paths are likely unsupported by the
        JVM or the game."""

    @property
    def strict_assets_check(self) -> bool:
        """When enabled, all assets are checked against their expected SHA-1, disabled by
        default because it's heavy on CPU."""
    @strict_assets_check.setter
    def strict_assets_check(self, strict: bool): ...

    @property
    def strict_libraries_check(self) -> bool:
        """When enabled, all libraries are checked against their expected SHA-1, disabled
        by default because it's heavy on CPU."""
    @strict_libraries_check.setter
    def strict_libraries_check(self, strict: bool): ...

    @property
    def strict_jvm_check(self) -> bool:
        """When enabled, all files of Mojang-provided JVMs are checked against their
        expected SHA-1, disabled by default because it's heavy on CPU."""
    @strict_jvm_check.setter
    def strict_jvm_check(self, strict: bool): ...

    @property
    def jvm_policy(self) -> Path | JvmPolicy:
        """The policy for finding a JVM to run the game, or the path to the JVM
        executable to use."""
    @jvm_policy.setter
    def jvm_policy(self, policy: str | PathLike[str] | JvmPolicy): ...

    @property
    def launcher_name(self) -> str:
        """The launcher name put on the command line, defaults to "portablemc"."""
    @launcher_name.setter
    def launcher_name(self, name: str): ...

    @property
    def launcher_version(self) -> str:
        """The launcher version put on the command line, defaults to PortableMC version."""
    @launcher_version.setter
    def launcher_version(self, name: str): ...

    def install(self, handler: Handler | None = None) -> Game:
        """Install the version and return the game ready to be launched. The GIL is
        released during the installation, the handler's methods are called from the
        calling thread. If the handler raises an exception, no other handler method
        is called, the installation is aborted as soon as possible and the exception
        is raised from this method."""


class JvmPolicy(Enum):
    """The policy for finding or installing the JVM used to launch the game."""

    System = auto()
    """Find a suitable JVM executable in the system's standard paths, raising
    `JvmNotFoundError` if no compatible one is found."""
    Mojang = auto()
    """Install a suitable Mojang-provided JVM, raising `JvmNotFoundError` if none is
    available for the platform and the required distribution."""
    SystemThenMojang = auto()
    """Search the system first, then fall back to Mojang."""
    MojangThenSystem = auto()
    """Search Mojang first, then fall back to the system."""


class Handler:
    """Handler for installation events, every method does nothing by default and can be
    overridden by subclasses. Methods are called by name, so any object with the
    relevant methods can be used as a handler."""

    def filter_features(self, features: set[str]) -> None:
        """Filter features used to resolve version rules, the set can be modified."""
    def loaded_features(self, features: set[str]) -> None:
        """The features selected after filtering."""
    def load_hierarchy(self, root_version: str) -> None:
        """The version hierarchy will be loaded, starting from the given root version."""
    def loaded_hierarchy(self, hierarchy: list[LoadedVersion]) -> None:
        """The version hierarchy has been loaded."""
    def load_version(self, version: str, file: Path) -> None:
        """A version will be loaded, the metadata file can be checked and deleted if
        invalid, in which case `need_version` is called just after."""
    def need_version(self, version: str, file: Path) -> bool:
        """The version metadata file is missing, the handler can install it and return
        true to retry loading it."""
    def loaded_version(self, version: str, file: Path) -> None:
        """A version of the hierarchy has been loaded from the given metadata file."""
    def load_client(self) -> None:
        """The client JAR file will be loaded."""
    def loaded_client(self, file: Path) -> None:
        """The client JAR file has been loaded at the given path."""
    def load_libraries(self) -> None:
        """The libraries required by the game will be loaded."""
    def filter_libraries(self, libraries: list[LoadedLibrary]) -> None:
        """Filter libraries before their verification, the list can be modified."""
    def loaded_libraries(self, libraries: list[LoadedLibrary]) -> None:
        """Libraries have been loaded, they will be verified and the missing ones
        downloaded."""
    def filter_libraries_files(self, class_files: list[Path], natives_files: list[Path]) -> None:
        """Filter the verified libraries files, the lists can be modified."""
    def loaded_libraries_files(self, class_files: list[Path], natives_files: list[Path]) -> None:
        """The final class and natives files have been loaded."""
    def no_logger(self) -> None:
        """No logger configuration is loaded because the version doesn't specify any."""
    def load_logger(self, id: str) -> None:
        """The logger configuration will be loaded."""
    def loaded_logger(self, id: str) -> None:
        """The logger configuration has been loaded."""
    def no_assets(self) -> None:
        """No assets are loaded because the version doesn't specify any."""
    def load_assets(self, id: str) -> None:
        """The assets will be loaded."""
    def loaded_assets(self, id: str, count: int) -> None:
        """The assets have been loaded and will be verified."""
    def verified_assets(self, id: str, count: int) -> None:
        """The assets have been verified and the missing ones will be downloaded."""
    def load_jvm(self, major_version: int) -> None:
        """The JVM will be loaded for the required major version, defaulting to 8 when
        not specified by the version."""
    def found_jvm_system_version(self, file: Path, version: str, compatible: bool) -> None:
        """A JVM executable has been found in the system, and is compatible or not."""
    def warn_jvm_unsupported_dynamic_crt(self) -> None:
        """The system has no dynamically linked C runtime (glibc), Mojang provides no
        JVM for such systems."""
    def warn_jvm_unsupported_platform(self) -> None:
        """Mojang provides no JVM for this operating system and architecture."""
    def warn_jvm_missing_distribution(self) -> None:
        """Mojang provides JVMs for this platform but not the required distribution."""
    def loaded_jvm(self, file: Path, version: str | None, compatible: bool) -> None:
        """The JVM has been loaded, with its version if known, compatible indicates if
        it is likely compatible with the game."""
    def download_resources(self) -> bool:
        """Resources will be downloaded, return true to cancel the download, the
        installation then fails with `DownloadResourcesCancelledError`."""
    def downloaded_resources(self) -> None:
        """Resources have been downloaded."""
    def download_progress(self, count: int, total_count: int, size: int, total_size: int) -> None:
        """Progress of the current download, in number of entries and in bytes."""
    def extracted_binaries(self, dir: Path) -> None:
        """All binaries have been extracted to the given directory."""


class VersionChannel(Enum):
    """The release channel of a version, called "type" by the game."""

    Release = auto()
    Snapshot = auto()
    Beta = auto()
    Alpha = auto()
    Unknown = auto()


class LoadedVersion:
    """A version loaded from the hierarchy."""

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str:
        """The version name, called "id" by the game."""
    @property
    def dir(self) -> Path:
        """The version directory, where its metadata and client JAR are stored."""
    @property
    def channel(self) -> VersionChannel | None:
        """The release channel, if specified in the metadata."""


class LoadedLibrary:
    """A library loaded from the version hierarchy."""

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
    def download(self) -> LibraryDownload | None:
        """How to download the library if it is missing."""
    @download.setter
    def download(self, download: LibraryDownload | None): ...

    @property
    def natives(self) -> bool:
        """True if the library contains natives to extract into the binaries directory,
        instead of being added to the class path."""
    @natives.setter
    def natives(self, natives: bool): ...


class LibraryDownload:
    """How to download a library, with optional expected size and SHA-1."""

    def __new__(cls, url: str, size: int | None = None, sha1: bytes | None = None) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def url(self) -> str: ...
    @property
    def size(self) -> int | None: ...
    @property
    def sha1(self) -> bytes | None: ...


class Game:
    """An installed game, ready to be launched."""

    def __repr__(self) -> str: ...

    jvm_file: Path
    """The JVM executable file."""
    mc_dir: Path
    """The working directory of the game process."""
    main_class: str
    """The main class containing the JVM entry point."""
    jvm_args: list[str]
    """The JVM arguments, before the main class in the command line."""
    game_args: list[str]
    """The game arguments, after the main class in the command line."""

    def args(self) -> list[str | Path]:
        """Return the full command line, starting with the JVM executable."""

    def command(self) -> partial[Popen[bytes]]:
        """Return a partial `subprocess.Popen` constructor with the command line and
        working directory already set, further arguments are given to `Popen`."""


def default_main_dir() -> Path | None:
    """Return the default main directory of the game (".minecraft"), if known for this
    operating system."""


class Error(Exception):
    """Base class for all installer errors."""

class HierarchyLoopError(Error):
    """A version appears twice in the hierarchy, implying an infinite loop."""
    version: str

class VersionNotFoundError(Error):
    """A version of the hierarchy is not found."""
    version: str

class AssetsNotFoundError(Error):
    """The assets index is not found and has no download information."""
    id: str

class ClientNotFoundError(Error):
    """The client JAR file is missing and has no download information."""

class LibraryNotFoundError(Error):
    """A library is missing and has no download information."""
    name: str

class JvmNotFoundError(Error):
    """No suitable JVM was found for the required major version, depending on the JVM
    policy."""
    major_version: int

class MainClassNotFoundError(Error):
    """No main class is specified by the version hierarchy."""

class DownloadResourcesCancelledError(Error):
    """The handler's `download_resources` method cancelled the download."""

class DownloadError(Error):
    """Some entries of the download failed."""
    errors: list[tuple[str, Path, str]]
    """The (url, file, reason) of each failed entry."""
    count: int
    """Total number of entries in the download."""

class InternalError(Error):
    """Unexpected internal error, the cause is set to an `OSError` for I/O errors."""
    origin: str
    """The location where the error originates."""
